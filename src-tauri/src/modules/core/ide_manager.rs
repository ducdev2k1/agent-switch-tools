use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::core::path_helpers::{ide_credential_source, ide_profiles_dir, ide_tools_dir};
use crate::modules::providers::antigravity::{self, CACHED_EMAIL_KEY, CACHED_NAME_KEY};
use crate::modules::providers::claude_cli::config::{read_meta, record_switch_usage, write_meta};
use crate::modules::providers::{IdeProvider, IdeType};

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

/// For variants that don't store identity locally (Antigravity IDE/CLI), resolve email+name
/// via Google userinfo (authoritative for the token's own account) and cache them into the
/// auth map so they persist into saved profiles. No-op for variants whose email is offline.
///
/// Note: `~/.gemini/google_accounts.json` is deliberately NOT used — it tracks the *Gemini*
/// CLI account, which can differ from the Antigravity CLI's logged-in account.
async fn ensure_identity(
    ide_type: &IdeType,
    provider: &dyn IdeProvider,
    data: &mut HashMap<String, String>,
) {
    if provider.extract_email(data).is_some() {
        return;
    }
    if let Some((email, name)) = antigravity::oauth::resolve_email_name(ide_type, data).await {
        data.insert(CACHED_EMAIL_KEY.to_string(), email);
        if let Some(n) = name {
            data.insert(CACHED_NAME_KEY.to_string(), n);
        }
    }
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
    
    let source = ide_credential_source(app, &ide_type)?;
    let tools_dir = ide_tools_dir(app, &ide_type)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;
    let meta = read_meta(&tools_dir);

    let mut profiles: Vec<IdeProfile> = vec![];
    if source.exists() {
        if let Ok(mut data) = source.read(provider.auth_keys()) {
            if !data.is_empty() {
                ensure_identity(&ide_type, provider.as_ref(), &mut data).await;
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
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(profiles)
}

pub async fn save_current_profile(app: &tauri::AppHandle, ide_type_str: &str) -> Result<String, String> {
    let ide_type = IdeType::from_str(ide_type_str)?;
    let provider = ide_type.provider();
    let source = ide_credential_source(app, &ide_type)?;
    let tools_dir = ide_tools_dir(app, &ide_type)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;

    let mut data = source.read(provider.auth_keys())?;
    ensure_identity(&ide_type, provider.as_ref(), &mut data).await;
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
    
    let source = ide_credential_source(app, &ide_type)?;
    let tools_dir = ide_tools_dir(app, &ide_type)?;
    let profs_dir = ide_profiles_dir(app, &ide_type)?;

    let ide_was_running = check_ide_running(&ide_type);
    let target_data = read_saved_auth_keys(&profs_dir, &target_name)?;
    let mut meta = read_meta(&tools_dir);

    if let Ok(mut curr) = source.read(provider.auth_keys()) {
        if !curr.is_empty() {
            ensure_identity(&ide_type, provider.as_ref(), &mut curr).await;
            let email = provider.extract_email(&curr).or_else(|| meta.active_profile_name.clone()).unwrap_or_default();
            if !email.is_empty() {
                let _ = write_saved_auth_keys(&profs_dir, &email, &curr);
                record_switch_usage(&mut meta, &email);
            }
        }
    }
    source.write(provider.auth_keys()[0], &target_data)?;
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
