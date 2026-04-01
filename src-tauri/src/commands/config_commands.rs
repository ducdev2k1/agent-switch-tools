use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

use super::metadata_commands::{read_meta, record_switch_usage, write_meta};

/// Resolve ~/.claude/ directory
fn claude_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .home_dir()
        .map(|h| h.join(".claude"))
        .map_err(|e| format!("Cannot resolve home directory: {}", e))
}

// ========== Types ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CredentialInfo {
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
    pub expires_at: Option<i64>,
    pub is_expired: bool,
    pub expires_in_hours: Option<f64>,
    pub scopes: Vec<String>,
    pub organization_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CredentialProfile {
    pub name: String,
    pub is_active: bool,
    pub info: CredentialInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchResult {
    pub success: bool,
    pub claude_was_running: bool,
    pub target_was_expired: bool,
    pub message: String,
}

/// Parse credential info from a .credentials JSON file
fn read_credential_info(path: &PathBuf) -> CredentialInfo {
    let default = CredentialInfo {
        subscription_type: None,
        rate_limit_tier: None,
        expires_at: None,
        is_expired: false,
        expires_in_hours: None,
        scopes: vec![],
        organization_uuid: None,
    };

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return default,
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return default,
    };

    let oauth = v.get("claudeAiOauth");
    let expires_at = oauth
        .and_then(|o| o.get("expiresAt"))
        .and_then(|s| s.as_i64());
    let now_ms = chrono::Utc::now().timestamp_millis();
    let is_expired = expires_at.map(|exp| exp < now_ms).unwrap_or(false);
    let expires_in_hours = expires_at.map(|exp| (exp - now_ms) as f64 / 3_600_000.0);

    CredentialInfo {
        subscription_type: oauth
            .and_then(|o| o.get("subscriptionType"))
            .and_then(|s| s.as_str())
            .map(String::from),
        rate_limit_tier: oauth
            .and_then(|o| o.get("rateLimitTier"))
            .and_then(|s| s.as_str())
            .map(String::from),
        expires_at,
        is_expired,
        expires_in_hours,
        scopes: oauth
            .and_then(|o| o.get("scopes"))
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        organization_uuid: v
            .get("organizationUuid")
            .and_then(|s| s.as_str())
            .map(String::from),
    }
}

/// Check if claude process is currently running
fn check_claude_running() -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", "claude"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ========== Commands ==========

/// List all credential profiles (active + saved)
#[tauri::command]
pub async fn list_credential_profiles(
    app: tauri::AppHandle,
) -> Result<Vec<CredentialProfile>, String> {
    let dir = claude_dir(&app)?;
    let meta = read_meta(&dir);
    let mut profiles: Vec<CredentialProfile> = vec![];

    // Active profile (.credentials.json)
    let active_path = dir.join(".credentials.json");
    if active_path.exists() {
        let active_name = meta
            .active_profile_name
            .clone()
            .unwrap_or_else(|| "Active".to_string());
        let info = read_credential_info(&active_path);
        profiles.push(CredentialProfile {
            name: active_name,
            is_active: true,
            info,
        });
    }

    // Saved profiles (.credentials-[name].json)
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with(".credentials-") && filename.ends_with(".json") {
                let name = filename
                    .strip_prefix(".credentials-")
                    .and_then(|s| s.strip_suffix(".json"))
                    .unwrap_or("")
                    .to_string();

                if !name.is_empty() {
                    let info = read_credential_info(&entry.path());
                    profiles.push(CredentialProfile {
                        name,
                        is_active: false,
                        info,
                    });
                }
            }
        }
    }

    // Sort: active first, then by name
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

/// Save current active credentials as a named profile (copy, not move)
#[tauri::command]
pub async fn save_current_as_profile(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let active_path = dir.join(".credentials.json");
    let target_path = dir.join(format!(".credentials-{}.json", name));

    if !active_path.exists() {
        return Err("No active credentials found (.credentials.json)".to_string());
    }
    if target_path.exists() {
        return Err(format!("Profile '{}' already exists", name));
    }

    std::fs::copy(&active_path, &target_path).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }

    // Update metadata: track this as the active profile name
    let mut meta = read_meta(&dir);
    meta.active_profile_name = Some(name);
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&dir, &meta)?;

    Ok(())
}

/// Switch to a different profile (atomic swap)
/// 1. Copy target → .credentials.json.tmp
/// 2. Rename active → .credentials-[current].json (backup)
/// 3. Rename .tmp → .credentials.json (activate)
/// 4. Update metadata
#[tauri::command]
pub async fn switch_credential_profile(
    app: tauri::AppHandle,
    target_name: String,
) -> Result<SwitchResult, String> {
    let dir = claude_dir(&app)?;
    let active_path = dir.join(".credentials.json");
    let tmp_path = dir.join(".credentials.json.tmp");
    let target_path = dir.join(format!(".credentials-{}.json", target_name));

    if !target_path.exists() {
        return Err(format!("Profile '{}' not found", target_name));
    }

    // Check target expiry
    let target_info = read_credential_info(&target_path);
    let target_was_expired = target_info.is_expired;

    // Check if Claude is running
    let claude_was_running = check_claude_running();

    // Read current active profile name from metadata
    let mut meta = read_meta(&dir);
    let current_name = meta
        .active_profile_name
        .clone()
        .unwrap_or_else(|| "Unnamed".to_string());

    // Step 1: Copy target to temp file
    std::fs::copy(&target_path, &tmp_path)
        .map_err(|e| format!("Failed to prepare target credentials: {}", e))?;

    // Step 2: Backup current active (if exists)
    if active_path.exists() {
        let backup_path = dir.join(format!(".credentials-{}.json", current_name));
        // Overwrite existing backup (update with latest refreshed tokens)
        std::fs::rename(&active_path, &backup_path)
            .map_err(|e| format!("Failed to backup current credentials: {}", e))?;
    }

    // Step 3: Activate target (rename tmp → active)
    std::fs::rename(&tmp_path, &active_path)
        .map_err(|e| format!("Failed to activate target credentials: {}", e))?;

    // Step 4: Remove the saved target file (now active)
    let _ = std::fs::remove_file(&target_path);

    // Step 5: Update metadata with usage tracking
    record_switch_usage(&mut meta, &current_name);
    meta.active_profile_name = Some(target_name.clone());
    write_meta(&dir, &meta)?;

    let message = if claude_was_running {
        "Switched credentials. Restart Claude Code to use new account.".to_string()
    } else if target_was_expired {
        "Switched to expired credentials. Token may auto-refresh on next use.".to_string()
    } else {
        format!("Switched to '{}'.", target_name)
    };

    Ok(SwitchResult {
        success: true,
        claude_was_running,
        target_was_expired,
        message,
    })
}

/// Rename a saved profile
#[tauri::command]
pub async fn rename_credential_profile(
    app: tauri::AppHandle,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let old_path = dir.join(format!(".credentials-{}.json", old_name));
    let new_path = dir.join(format!(".credentials-{}.json", new_name));

    if !old_path.exists() {
        return Err(format!("Profile '{}' not found", old_name));
    }
    if new_path.exists() {
        return Err(format!("Profile '{}' already exists", new_name));
    }

    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete a saved profile
#[tauri::command]
pub async fn delete_credential_profile(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let path = dir.join(format!(".credentials-{}.json", name));

    if !path.exists() {
        return Err(format!("Profile '{}' not found", name));
    }

    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Check if Claude Code CLI process is running
#[tauri::command]
pub async fn is_claude_running() -> Result<bool, String> {
    Ok(check_claude_running())
}

// ========== CLI State ==========

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCliState {
    pub current_model: Option<String>,
    pub session_count: usize,
    pub env_file_exists: bool,
    pub active_keys: Vec<String>,
}

#[tauri::command]
pub async fn get_claude_cli_state(app: tauri::AppHandle) -> Result<ClaudeCliState, String> {
    let dir = claude_dir(&app)?;
    let current_model = read_settings_model(&dir);
    let session_count = count_history_sessions(&dir);
    let env_path = dir.join(".env");
    let env_file_exists = env_path.exists();
    let active_keys = if env_file_exists {
        read_env_key_names(&env_path)
    } else {
        vec![]
    };

    Ok(ClaudeCliState {
        current_model,
        session_count,
        env_file_exists,
        active_keys,
    })
}

fn read_settings_model(dir: &PathBuf) -> Option<String> {
    let path = dir.join("settings.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("model")?.as_str().map(|s| s.to_string())
}

fn count_history_sessions(dir: &PathBuf) -> usize {
    let path = dir.join("history.jsonl");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .count()
}

fn read_env_key_names(env_path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(env_path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.starts_with('#') && l.contains('='))
        .map(|l| l.split('=').next().unwrap_or("").trim().to_string())
        .filter(|k| !k.is_empty())
        .collect()
}
