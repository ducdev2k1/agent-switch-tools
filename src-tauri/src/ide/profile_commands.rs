use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::path_helpers::{ide_db_path, ide_profiles_dir, ide_tools_dir};
use super::registry::IdeType;
use super::sqlite_auth::{extract_ide_email, read_ide_auth_keys, write_ide_auth_keys};
use crate::commands::metadata_commands::{read_meta, record_switch_usage, write_meta};

// ========== Validation ==========

/// Sanitize profile name to prevent path traversal attacks.
/// Rejects names containing path separators, "..", or starting with ".".
fn sanitize_profile_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.starts_with('.')
    {
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

/// Read saved auth-keys.json from a profile directory
fn read_saved_auth_keys(
    profiles_dir: &std::path::Path,
    name: &str,
) -> Result<HashMap<String, String>, String> {
    let path = profiles_dir.join(name).join("auth-keys.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profile '{}': {}", name, e))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid profile data '{}': {}", name, e))
}

/// Write auth-keys.json to a profile directory
fn write_saved_auth_keys(
    profiles_dir: &std::path::Path,
    name: &str,
    data: &HashMap<String, String>,
) -> Result<(), String> {
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

/// Extract extra display info from auth data per IDE type
/// Returns (display_name, membership_type)
fn extract_profile_display_info(
    ide_type: &IdeType,
    auth_data: &HashMap<String, String>,
) -> (Option<String>, Option<String>) {
    match ide_type {
        IdeType::Cursor => {
            let membership = auth_data
                .get("cursorAuth/stripeMembershipType")
                .cloned();
            (None, membership)
        }
        IdeType::Antigravity => {
            let (display_name, membership) = auth_data
                .get("antigravityAuthStatus")
                .and_then(|json_str| {
                    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
                    let name = v.get("name")?.as_str().map(String::from);
                    let plan = extract_plan_from_proto_json(&v);
                    Some((name, plan))
                })
                .unwrap_or((None, None));
            (display_name, membership)
        }
        IdeType::Windsurf => {
            // Display name stored in "codeium.windsurf-windsurf_auth" key
            let display_name = auth_data
                .get("codeium.windsurf-windsurf_auth")
                .cloned()
                .filter(|s| !s.is_empty() && s != "[]");
            // Plan info from protobuf in windsurfAuthStatus
            let membership = auth_data
                .get("windsurfAuthStatus")
                .and_then(|json_str| {
                    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
                    extract_plan_from_proto_json(&v)
                });
            (display_name, membership)
        }
    }
}

/// Extract plan/tier name from JSON that contains userStatusProtoBinaryBase64
/// The protobuf stores plan name (e.g. "Free", "Pro", "Team") as readable text
fn extract_plan_from_proto_json(json: &serde_json::Value) -> Option<String> {
    let b64 = json.get("userStatusProtoBinaryBase64")?.as_str()?;
    let decoded = data_encoding::BASE64.decode(b64.as_bytes()).ok()?;
    let text = String::from_utf8_lossy(&decoded);
    // Known plan names in Codeium/Windsurf/Antigravity ecosystem
    let known_plans = ["Enterprise", "Team", "Pro Ultimate", "Pro", "Free"];
    for plan in &known_plans {
        if text.contains(plan) {
            return Some(plan.to_string());
        }
    }
    None
}

/// Check if an IDE process is currently running (cross-platform)
fn check_ide_running(ide_type: &IdeType) -> bool {
    let config = ide_type.config();

    #[cfg(unix)]
    {
        for process_name in config.process_names {
            let is_running = std::process::Command::new("pgrep")
                .args(["-f", process_name])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if is_running {
                return true;
            }
        }
        false
    }

    #[cfg(windows)]
    {
        for process_name in config.process_names {
            let is_running = std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}.exe", process_name)])
                .output()
                .map(|o| {
                    String::from_utf8_lossy(&o.stdout)
                        .to_lowercase()
                        .contains(&process_name.to_lowercase())
                })
                .unwrap_or(false);
            if is_running {
                return true;
            }
        }
        false
    }
}

// ========== Tauri Commands ==========

/// List all profiles for a specific IDE (active + saved)
#[tauri::command]
pub async fn list_ide_profiles(
    app: tauri::AppHandle,
    ide_type: String,
) -> Result<Vec<IdeProfile>, String> {
    let ide = IdeType::from_str(&ide_type)?;
    let config = ide.config();
    let db_path = ide_db_path(&app, &ide)?;
    let tools_dir = ide_tools_dir(&app, &ide)?;
    let profs_dir = ide_profiles_dir(&app, &ide)?;
    let meta = read_meta(&tools_dir);

    let mut profiles: Vec<IdeProfile> = vec![];

    // Read active account from IDE's state.vscdb (if accessible)
    if db_path.exists() {
        if let Ok(auth_data) = read_ide_auth_keys(&db_path, config.auth_keys) {
            if !auth_data.is_empty() {
                let email = extract_ide_email(&ide, &auth_data);
                // Use email from DB as primary identifier for the active account
                // This ensures accuracy even if user manually logged out/in via IDE
                let active_name = email
                    .clone()
                    .or_else(|| meta.active_profile_name.clone())
                    .unwrap_or_else(|| "Active".to_string());
                let (display_name, membership_type) =
                    extract_profile_display_info(&ide, &auth_data);

                // Auto-sync meta if user changed account externally (drift detection)
                if let Some(ref db_email) = email {
                    if meta.active_profile_name.as_ref() != Some(db_email) {
                        let mut updated_meta = meta.clone();
                        updated_meta.active_profile_name = Some(db_email.clone());
                        let _ = write_meta(&tools_dir, &updated_meta);
                    }
                }

                profiles.push(IdeProfile {
                    name: active_name,
                    is_active: true,
                    email,
                    membership_type,
                    display_name,
                    ide_type: ide_type.clone(),
                });
            }
        }
    }

    // Scan saved profiles
    if let Ok(entries) = std::fs::read_dir(&profs_dir) {
        // Compare saved profile name with current active DB email (not stale meta)
        let current_active_email = profiles.first().and_then(|p| p.email.clone());
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.is_empty() || Some(&name) == current_active_email.as_ref() {
                continue;
            }
            let auth_keys_path = entry.path().join("auth-keys.json");
            if !auth_keys_path.exists() {
                continue;
            }
            // Read saved auth data for display info
            let (email, display_name, membership_type) =
                if let Ok(saved_data) = read_saved_auth_keys(&profs_dir, &name) {
                    let email = extract_ide_email(&ide, &saved_data);
                    let (dn, mt) = extract_profile_display_info(&ide, &saved_data);
                    (email, dn, mt)
                } else {
                    (Some(name.clone()), None, None)
                };

            profiles.push(IdeProfile {
                name,
                is_active: false,
                email,
                membership_type,
                display_name,
                ide_type: ide_type.clone(),
            });
        }
    }

    // Sort: active first, then alphabetical
    profiles.sort_by(|a, b| {
        if a.is_active {
            std::cmp::Ordering::Less
        } else if b.is_active {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(profiles)
}

/// Save the current active IDE account as a named profile
#[tauri::command]
pub async fn save_current_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
) -> Result<String, String> {
    let ide = IdeType::from_str(&ide_type)?;
    let config = ide.config();
    let db_path = ide_db_path(&app, &ide)?;
    let tools_dir = ide_tools_dir(&app, &ide)?;
    let profs_dir = ide_profiles_dir(&app, &ide)?;

    let auth_data = read_ide_auth_keys(&db_path, config.auth_keys)?;
    if auth_data.is_empty() {
        return Err(format!(
            "No account is currently logged in to {}",
            config.display_name
        ));
    }

    let email = extract_ide_email(&ide, &auth_data)
        .ok_or_else(|| format!("Cannot detect email from {} auth data", config.display_name))?;
    let email = sanitize_profile_name(&email)?;

    // Save auth keys to profile directory
    write_saved_auth_keys(&profs_dir, &email, &auth_data)?;

    // Update metadata
    let mut meta = read_meta(&tools_dir);
    meta.active_profile_name = Some(email.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&tools_dir, &meta)?;

    crate::tray::refresh_tray_menu(&app);
    Ok(email)
}

/// Switch to a different IDE profile
/// 1. Check if IDE is running (warn if yes)
/// 2. Backup current auth keys to their profile dir
/// 3. Read target auth-keys.json
/// 4. Write target keys into state.vscdb
/// 5. Update metadata
#[tauri::command]
pub async fn switch_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
    target_name: String,
) -> Result<IdeSwitchResult, String> {
    let target_name = sanitize_profile_name(&target_name)?;
    let ide = IdeType::from_str(&ide_type)?;
    let config = ide.config();
    let db_path = ide_db_path(&app, &ide)?;
    let tools_dir = ide_tools_dir(&app, &ide)?;
    let profs_dir = ide_profiles_dir(&app, &ide)?;

    let ide_was_running = check_ide_running(&ide);

    // Read target profile
    let target_data = read_saved_auth_keys(&profs_dir, &target_name)?;

    // Backup current auth keys before switching
    let mut meta = read_meta(&tools_dir);
    if let Ok(current_data) = read_ide_auth_keys(&db_path, config.auth_keys) {
        if !current_data.is_empty() {
            let current_email = meta
                .active_profile_name
                .clone()
                .or_else(|| extract_ide_email(&ide, &current_data))
                .unwrap_or_default();

            if !current_email.is_empty() {
                let _ = write_saved_auth_keys(&profs_dir, &current_email, &current_data);
                record_switch_usage(&mut meta, &current_email);
            }
        }
    }

    // Write target auth keys into IDE's state.vscdb
    write_ide_auth_keys(&db_path, &target_data)?;

    // Update metadata
    meta.active_profile_name = Some(target_name.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&tools_dir, &meta)?;

    let message = if ide_was_running {
        format!(
            "Switched to '{}'. Restart {} to use new account.",
            target_name, config.display_name
        )
    } else {
        format!("Switched to '{}' in {}.", target_name, config.display_name)
    };

    crate::tray::refresh_tray_menu(&app);
    Ok(IdeSwitchResult {
        success: true,
        ide_was_running,
        message,
    })
}

/// Rename a saved IDE profile directory
#[tauri::command]
pub async fn rename_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let old_name = sanitize_profile_name(&old_name)?;
    let new_name = sanitize_profile_name(&new_name)?;
    let ide = IdeType::from_str(&ide_type)?;
    let profs_dir = ide_profiles_dir(&app, &ide)?;
    let old_dir = profs_dir.join(&old_name);
    let new_dir = profs_dir.join(&new_name);

    if !old_dir.exists() {
        return Err(format!("Profile '{}' not found", old_name));
    }
    if new_dir.exists() {
        return Err(format!("Profile '{}' already exists", new_name));
    }

    std::fs::rename(&old_dir, &new_dir).map_err(|e| e.to_string())?;
    crate::tray::refresh_tray_menu(&app);
    Ok(())
}

/// Delete a saved IDE profile directory
#[tauri::command]
pub async fn delete_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
    name: String,
) -> Result<(), String> {
    let name = sanitize_profile_name(&name)?;
    let ide = IdeType::from_str(&ide_type)?;
    let profs_dir = ide_profiles_dir(&app, &ide)?;
    let prof_dir = profs_dir.join(&name);

    if !prof_dir.exists() {
        return Err(format!("Profile '{}' not found", name));
    }

    std::fs::remove_dir_all(&prof_dir).map_err(|e| e.to_string())?;
    crate::tray::refresh_tray_menu(&app);
    Ok(())
}

/// Check if a specific IDE process is currently running
#[tauri::command]
pub async fn is_ide_running(ide_type: String) -> Result<bool, String> {
    let ide = IdeType::from_str(&ide_type)?;
    Ok(check_ide_running(&ide))
}

/// Restart an IDE by killing its process and relaunching via CLI
#[tauri::command]
pub async fn restart_ide(ide_type: String) -> Result<String, String> {
    let ide = IdeType::from_str(&ide_type)?;
    let config = ide.config();

    if !check_ide_running(&ide) {
        return Err(format!("{} is not running", config.display_name));
    }

    // Kill the IDE process gracefully
    #[cfg(unix)]
    {
        for process_name in config.process_names {
            let _ = std::process::Command::new("pkill")
                .args(["-f", process_name])
                .output();
        }
    }

    #[cfg(windows)]
    {
        for process_name in config.process_names {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/IM", &format!("{}.exe", process_name)])
                .output();
        }
    }

    // Wait for process to fully exit
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Relaunch IDE via CLI (detached)
    let launch_result = std::process::Command::new(config.cli_command)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match launch_result {
        Ok(_) => Ok(format!("{} is restarting...", config.display_name)),
        Err(e) => Err(format!(
            "Killed {} but failed to relaunch: {}",
            config.display_name, e
        )),
    }
}
