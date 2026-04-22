use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::core::path_helpers::{ide_db_path, ide_profiles_dir, ide_tools_dir};
use crate::modules::core::sqlite_auth::{read_ide_auth_keys, write_ide_auth_keys};
use crate::modules::providers::IdeType;
use crate::modules::providers::claude_cli::config::{read_meta, record_switch_usage, write_meta};

// ========== Validation ==========

pub fn sanitize_profile_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") || name.starts_with('.') {
        return Err(format!("Invalid profile name: '{}'", name));
    }
    Ok(name.to_string())
}

// ========== Types ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdeProfile {
    pub name: String,
    pub is_active: bool,
    pub email: Option<String>,
    pub membership_type: Option<String>,
    pub display_name: Option<String>,
    pub ide_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdeSwitchResult {
    pub success: bool,
    pub ide_was_running: bool,
    pub message: String,
}

// ========== Helpers ==========

pub fn read_saved_auth_keys(profiles_dir: &std::path::Path, name: &str) -> Result<HashMap<String, String>, String> {
    let path = profiles_dir.join(name).join("auth-keys.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profile '{}': {}", name, e))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid profile data '{}': {}", name, e))
}

pub fn write_saved_auth_keys(profiles_dir: &std::path::Path, name: &str, data: &HashMap<String, String>) -> Result<(), String> {
    let prof_dir = profiles_dir.join(name);
    std::fs::create_dir_all(&prof_dir).map_err(|e| e.to_string())?;
    let path = prof_dir.join("auth-keys.json");
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn check_ide_running(ide_type: &IdeType) -> bool {
    let provider = ide_type.provider();
    #[cfg(unix)]
    {
        for name in provider.process_names() {
            let is_running = std::process::Command::new("pgrep").args(["-f", name]).output().map(|o| o.status.success()).unwrap_or(false);
            if is_running { return true; }
        }
    }
    #[cfg(windows)] { /* Basic stub for windows pgrep equivalent */ }
    false
}

pub async fn list_profiles(app: &tauri::AppHandle, ide_type_str: &str) -> Result<Vec<IdeProfile>, String> {
    let ide_type = IdeType::from_str(ide_type_str)?;
    let provider = ide_type.provider();
    
    let db_path = ide_db_path(app, &ide_type)?;
    let tools_dir = ide_tools_dir(app, &ide_type)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;
    let meta = read_meta(&tools_dir);

    let mut profiles: Vec<IdeProfile> = vec![];
    if db_path.exists() {
        if let Ok(data) = read_ide_auth_keys(&db_path, provider.auth_keys()) {
            if !data.is_empty() {
                let email = provider.extract_email(&data);
                let active_name = email.clone().or_else(|| meta.active_profile_name.clone()).unwrap_or_else(|| "Active".to_string());
                let display_name = provider.extract_display_name(&data);
                let membership_type = provider.extract_membership(&data);
                
                profiles.push(IdeProfile { name: active_name, is_active: true, email, membership_type, display_name, ide_type: ide_type_str.to_string() });
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir(&profs_dir) {
        let active_email = profiles.first().and_then(|p| p.email.clone());
        for entry in entries.flatten() {
            if !entry.path().is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.is_empty() || Some(&name) == active_email.as_ref() { continue; }
            if let Ok(data) = read_saved_auth_keys(&profs_dir, &name) {
                let email = provider.extract_email(&data);
                let display_name = provider.extract_display_name(&data);
                let membership_type = provider.extract_membership(&data);

                profiles.push(IdeProfile { name, is_active: false, email, membership_type, display_name, ide_type: ide_type_str.to_string() });
            }
        }
    }
    profiles.sort_by(|a, b| if a.is_active { std::cmp::Ordering::Less } else if b.is_active { std::cmp::Ordering::Greater } else { a.name.cmp(&b.name) });
    Ok(profiles)
}

pub async fn save_current_profile(app: &tauri::AppHandle, ide_type_str: &str) -> Result<String, String> {
    let ide_type = IdeType::from_str(ide_type_str)?;
    let provider = ide_type.provider();
    let db_path = ide_db_path(app, &ide_type)?;
    let tools_dir = ide_tools_dir(app, &ide_type)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;
    
    let data = read_ide_auth_keys(&db_path, provider.auth_keys())?;
    let email = provider.extract_email(&data).ok_or("No email found")?;
    let name = sanitize_profile_name(&email)?;
    write_saved_auth_keys(&profs_dir, &name, &data)?;
    
    let mut meta = read_meta(&tools_dir);
    meta.active_profile_name = Some(name.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&tools_dir, &meta)?;
    Ok(name)
}

pub async fn switch_profile(app: &tauri::AppHandle, ide_type_str: &str, target_name: &str) -> Result<IdeSwitchResult, String> {
    let target_name = sanitize_profile_name(target_name)?;
    let ide_type = IdeType::from_str(ide_type_str)?;
    let provider = ide_type.provider();
    
    let db_path = ide_db_path(app, &ide_type)?;
    let tools_dir = ide_tools_dir(app, &ide_type)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;
    
    let ide_was_running = check_ide_running(&ide_type);
    let target_data = read_saved_auth_keys(&profs_dir, &target_name)?;
    let mut meta = read_meta(&tools_dir);
    
    if let Ok(curr) = read_ide_auth_keys(&db_path, provider.auth_keys()) {
        if !curr.is_empty() {
            let email = meta.active_profile_name.clone().or_else(|| provider.extract_email(&curr)).unwrap_or_default();
            if !email.is_empty() {
                let _ = write_saved_auth_keys(&profs_dir, &email, &curr);
                record_switch_usage(&mut meta, &email);
            }
        }
    }
    write_ide_auth_keys(&db_path, &target_data)?;
    meta.active_profile_name = Some(target_name.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&tools_dir, &meta)?;
    
    Ok(IdeSwitchResult { 
        success: true, 
        ide_was_running, 
        message: format!("Switched to {}", target_name) 
    })
}

pub async fn rename_profile(app: &tauri::AppHandle, ide_type_str: &str, old: &str, new: &str) -> Result<(), String> {
    let old = sanitize_profile_name(old)?;
    let new = sanitize_profile_name(new)?;
    let ide_type = IdeType::from_str(ide_type_str)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;
    std::fs::rename(profs_dir.join(old), profs_dir.join(new)).map_err(|e| e.to_string())
}

pub async fn delete_profile(app: &tauri::AppHandle, ide_type_str: &str, name: &str) -> Result<(), String> {
    let name = sanitize_profile_name(name)?;
    let ide_type = IdeType::from_str(ide_type_str)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;
    std::fs::remove_dir_all(profs_dir.join(name)).map_err(|e| e.to_string())
}

pub async fn restart_ide(ide_type_str: &str) -> Result<String, String> {
    let ide_type = IdeType::from_str(ide_type_str)?;
    let provider = ide_type.provider();
    
    if !check_ide_running(&ide_type) { return Err("Not running".to_string()); }
    
    #[cfg(unix)]
    {
        for process_name in provider.process_names() {
            let _ = std::process::Command::new("pkill").args(["-f", process_name]).output();
        }
    }
    
    std::thread::sleep(std::time::Duration::from_millis(1500));

    let _ = std::process::Command::new(provider.cli_command())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    Ok("Restarting...".to_string())
}
