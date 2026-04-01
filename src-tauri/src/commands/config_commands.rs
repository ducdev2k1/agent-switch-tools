use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

use super::metadata_commands::{read_meta, record_switch_usage, write_meta};
use super::oauth_commands::{
    read_oauth_from_claude_json, read_saved_oauth, update_claude_json_oauth, write_saved_oauth,
    OAuthAccount,
};

/// Resolve home directory (~)
fn home_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .home_dir()
        .map_err(|e| format!("Cannot resolve home directory: {}", e))
}

/// Resolve ~/.claude/ directory
fn claude_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    home_dir(app).map(|h| h.join(".claude"))
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
    pub oauth_account: Option<OAuthAccount>,
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
    let home = home_dir(&app)?;
    let dir = claude_dir(&app)?;
    let meta = read_meta(&dir);
    let mut profiles: Vec<CredentialProfile> = vec![];

    // Read active oauthAccount from ~/.claude.json
    let active_oauth = read_oauth_from_claude_json(&home);

    // Resolve active profile name: metadata → oauthAccount email → fallback
    let active_name = meta
        .active_profile_name
        .clone()
        .or_else(|| {
            active_oauth
                .as_ref()
                .and_then(|o| o.email_address.clone())
        })
        .unwrap_or_else(|| "Active".to_string());

    // Active profile (.credentials.json)
    let active_path = dir.join(".credentials.json");
    if active_path.exists() {
        let info = read_credential_info(&active_path);
        profiles.push(CredentialProfile {
            name: active_name.clone(),
            is_active: true,
            info,
            oauth_account: active_oauth,
        });
    }

    // Saved profiles (.credentials-[name].json), skip active
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with(".credentials-") && filename.ends_with(".json") {
                let name = filename
                    .strip_prefix(".credentials-")
                    .and_then(|s| s.strip_suffix(".json"))
                    .unwrap_or("")
                    .to_string();

                // Skip empty names and the currently active profile
                if name.is_empty() || name == active_name {
                    continue;
                }

                let info = read_credential_info(&entry.path());
                let oauth = read_saved_oauth(&dir, &name);
                profiles.push(CredentialProfile {
                    name,
                    is_active: false,
                    info,
                    oauth_account: oauth,
                });
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

/// Save current active credentials using email from oauthAccount (auto-detect)
#[tauri::command]
pub async fn save_current_as_profile(app: tauri::AppHandle) -> Result<String, String> {
    let home = home_dir(&app)?;
    let dir = claude_dir(&app)?;
    let active_path = dir.join(".credentials.json");

    if !active_path.exists() {
        return Err("No active credentials found (.credentials.json)".to_string());
    }

    // Auto-detect email from oauthAccount
    let oauth = read_oauth_from_claude_json(&home)
        .ok_or("Cannot read oauthAccount from ~/.claude.json")?;
    let email = oauth
        .email_address
        .clone()
        .ok_or("No email address found in oauthAccount")?;

    // Copy credentials (overwrite allowed — refreshes tokens)
    let target_path = dir.join(format!(".credentials-{}.json", email));
    std::fs::copy(&active_path, &target_path).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }

    // Save oauthAccount as .claude-[email].json
    write_saved_oauth(&dir, &email, &oauth)?;

    // Update metadata
    let mut meta = read_meta(&dir);
    meta.active_profile_name = Some(email.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&dir, &meta)?;

    Ok(email)
}

/// Switch to a different profile
/// 1. Backup current credentials + oauthAccount
/// 2. Copy target → .credentials.json (keep target file)
/// 3. Restore target's oauthAccount into ~/.claude.json
/// 4. Update metadata
#[tauri::command]
pub async fn switch_credential_profile(
    app: tauri::AppHandle,
    target_name: String,
) -> Result<SwitchResult, String> {
    let home = home_dir(&app)?;
    let dir = claude_dir(&app)?;
    let active_path = dir.join(".credentials.json");
    let target_path = dir.join(format!(".credentials-{}.json", target_name));

    if !target_path.exists() {
        return Err(format!("Profile '{}' not found", target_name));
    }

    let target_info = read_credential_info(&target_path);
    let target_was_expired = target_info.is_expired;
    let claude_was_running = check_claude_running();

    // Resolve outgoing email: metadata → oauthAccount → empty
    let mut meta = read_meta(&dir);
    let current_email = meta
        .active_profile_name
        .clone()
        .or_else(|| {
            read_oauth_from_claude_json(&home).and_then(|o| o.email_address)
        })
        .unwrap_or_default();

    // Backup outgoing: save credentials + oauthAccount
    if active_path.exists() && !current_email.is_empty() {
        let backup_path = dir.join(format!(".credentials-{}.json", current_email));
        std::fs::copy(&active_path, &backup_path)
            .map_err(|e| format!("Failed to backup current credentials: {}", e))?;

        if let Some(oauth) = read_oauth_from_claude_json(&home) {
            let _ = write_saved_oauth(&dir, &current_email, &oauth);
        }
    }

    // Activate target: copy (keep original file intact)
    std::fs::copy(&target_path, &active_path)
        .map_err(|e| format!("Failed to activate target credentials: {}", e))?;

    // Restore target's oauthAccount into ~/.claude.json
    if let Some(target_oauth) = read_saved_oauth(&dir, &target_name) {
        let _ = update_claude_json_oauth(&home, &target_oauth);
    }

    // Update metadata with usage tracking
    if !current_email.is_empty() {
        record_switch_usage(&mut meta, &current_email);
    }
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

/// Delete a saved profile (credentials + oauthAccount files)
#[tauri::command]
pub async fn delete_credential_profile(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let cred_path = dir.join(format!(".credentials-{}.json", name));

    if !cred_path.exists() {
        return Err(format!("Profile '{}' not found", name));
    }

    std::fs::remove_file(&cred_path).map_err(|e| e.to_string())?;

    // Also remove oauthAccount file if exists
    let oauth_path = dir.join(format!(".claude-{}.json", name));
    let _ = std::fs::remove_file(&oauth_path);

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
