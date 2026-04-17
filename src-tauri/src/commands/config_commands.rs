use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::metadata_commands::{read_meta, record_switch_usage, write_meta};
use super::oauth_commands::{
    read_oauth_from_claude_json, read_saved_oauth, update_claude_json_oauth, write_saved_oauth,
    OAuthAccount,
};
use super::path_helpers::{claude_data_dir, claude_dir, home_dir, profile_dir, profiles_dir};

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

// ========== Helpers ==========

/// Parse credential info from a credentials JSON file
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

/// Set file permissions to 600 (owner read/write only)
#[cfg(unix)]
fn set_file_600(path: &PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn set_file_600(_path: &PathBuf) {}

/// One-time migration: move profile files into ~/.agent-switch-tools/claude/profiles/
/// Safe to call repeatedly — skips files that already exist at destination.
///
/// Handles four legacy sources (in priority order):
///   1. ~/.agent-switch-tools/profiles/ (flat structure before claude/ subdir)
///   2. ~/.claude/.claude-tools/        (Option B path, previous nested structure)
///   3. ~/.claude-tools/                (Option A path, intermediate structure)
///   4. ~/.claude/                      (original flat files before any migration)
fn migrate_legacy_profiles(claude: &PathBuf, profs_dir: &PathBuf) {
    // profs_dir = ~/.agent-switch-tools/claude/profiles/
    // claude_data = ~/.agent-switch-tools/claude/
    let claude_data = match profs_dir.parent() {
        Some(d) => d.to_path_buf(),
        None => return,
    };

    // Helper: migrate profiles from a legacy root dir into the current claude_data
    fn migrate_from_legacy_root(legacy_root: &PathBuf, claude_data: &PathBuf, profs_dir: &PathBuf) {
        if !legacy_root.exists() {
            return;
        }
        // Move meta.json only if dest doesn't exist
        let src_meta = legacy_root.join("meta.json");
        let dst_meta = claude_data.join("meta.json");
        if src_meta.exists() && !dst_meta.exists() {
            let _ = std::fs::rename(&src_meta, &dst_meta);
        }

        // Move each profile dir
        let src_profiles = legacy_root.join("profiles");
        if let Ok(entries) = std::fs::read_dir(&src_profiles) {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let dst_prof = profs_dir.join(&name);
                let _ = std::fs::create_dir_all(&dst_prof);

                // Move credentials.json if not yet at destination
                let src_cred = entry.path().join("credentials.json");
                let dst_cred = dst_prof.join("credentials.json");
                if src_cred.exists() && !dst_cred.exists() {
                    let _ = std::fs::rename(&src_cred, &dst_cred);
                }

                // Move oauth.json if not yet at destination
                let src_oauth = entry.path().join("oauth.json");
                let dst_oauth = dst_prof.join("oauth.json");
                if src_oauth.exists() && !dst_oauth.exists() {
                    let _ = std::fs::rename(&src_oauth, &dst_oauth);
                }

                // Only remove src profile dir if it is now empty
                let _ = std::fs::remove_dir(&entry.path());
            }
        }

        // Only remove src dirs if empty — never force-delete
        let _ = std::fs::remove_dir(&src_profiles);
        let _ = std::fs::remove_dir(legacy_root);
    }

    // --- Phase 1: migrate from ~/.agent-switch-tools/profiles/ (flat → claude/ subdir) ---
    // This handles the case where profiles were at ~/.agent-switch-tools/profiles/ before
    // we introduced the claude/ subdirectory
    if let Some(app_root) = claude_data.parent() {
        let flat_profiles = app_root.join("profiles");
        if flat_profiles.exists() && flat_profiles.is_dir() {
            // Move meta.json from app root to claude_data
            let src_meta = app_root.join("meta.json");
            let dst_meta = claude_data.join("meta.json");
            if src_meta.exists() && !dst_meta.exists() {
                let _ = std::fs::rename(&src_meta, &dst_meta);
            }

            if let Ok(entries) = std::fs::read_dir(&flat_profiles) {
                for entry in entries.flatten() {
                    if !entry.path().is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Skip IDE subdirectories (they have their own structure)
                    if name == "claude" {
                        continue;
                    }
                    let dst_prof = profs_dir.join(&name);
                    let _ = std::fs::create_dir_all(&dst_prof);

                    let src_cred = entry.path().join("credentials.json");
                    let dst_cred = dst_prof.join("credentials.json");
                    if src_cred.exists() && !dst_cred.exists() {
                        let _ = std::fs::rename(&src_cred, &dst_cred);
                    }

                    let src_oauth = entry.path().join("oauth.json");
                    let dst_oauth = dst_prof.join("oauth.json");
                    if src_oauth.exists() && !dst_oauth.exists() {
                        let _ = std::fs::rename(&src_oauth, &dst_oauth);
                    }

                    let _ = std::fs::remove_dir(&entry.path());
                }
            }
            let _ = std::fs::remove_dir(&flat_profiles);
        }
    }

    // --- Phase 2: migrate from ~/.claude/.claude-tools/ (Option B → new location) ---
    let option_b_root = claude.join(".claude-tools");
    migrate_from_legacy_root(&option_b_root, &claude_data, profs_dir);

    // --- Phase 3: migrate from ~/.claude-tools/ (Option A → new location) ---
    if let Some(home) = claude.parent() {
        let option_a_root = home.join(".claude-tools");
        migrate_from_legacy_root(&option_a_root, &claude_data, profs_dir);
    }

    // --- Phase 4: migrate flat files from ~/.claude/ (original legacy format) ---
    // Migrate meta: ~/.claude/.claude-manager-meta.json
    let old_meta = claude.join(".claude-manager-meta.json");
    if old_meta.exists() {
        let new_meta = claude_data.join("meta.json");
        if !new_meta.exists() {
            let _ = std::fs::rename(&old_meta, &new_meta);
        }
    }

    let entries = match std::fs::read_dir(claude) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();

        // .credentials-{name}.json → profiles/{name}/credentials.json
        if let Some(name) = filename
            .strip_prefix(".credentials-")
            .and_then(|s| s.strip_suffix(".json"))
        {
            if !name.is_empty() {
                let prof_dir = profs_dir.join(name);
                let dest = prof_dir.join("credentials.json");
                if path.exists() && !dest.exists() {
                    let _ = std::fs::create_dir_all(&prof_dir);
                    if std::fs::rename(&path, &dest).is_ok() {
                        set_file_600(&dest);
                    }
                }
            }
            continue;
        }

        // .claude-{name}.json → profiles/{name}/oauth.json (skip manager meta)
        if filename != ".claude-manager-meta.json" {
            if let Some(name) = filename
                .strip_prefix(".claude-")
                .and_then(|s| s.strip_suffix(".json"))
            {
                if !name.is_empty() {
                    let prof_dir = profs_dir.join(name);
                    let dest = prof_dir.join("oauth.json");
                    if path.exists() && !dest.exists() {
                        let _ = std::fs::create_dir_all(&prof_dir);
                        if std::fs::rename(&path, &dest).is_ok() {
                            set_file_600(&dest);
                        }
                    }
                }
            }
        }
    }
}

// ========== Commands ==========

/// List all credential profiles (active + saved)
#[tauri::command]
pub async fn list_credential_profiles(
    app: tauri::AppHandle,
) -> Result<Vec<CredentialProfile>, String> {
    let home = home_dir(&app)?;
    let claude = claude_dir(&app)?;
    let claude_data = claude_data_dir(&app)?;
    let profs_dir = profiles_dir(&app)?;
    let meta = read_meta(&claude_data);

    // Migrate legacy files on first use
    migrate_legacy_profiles(&claude, &profs_dir);

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

    // Active profile: ~/.claude/.credentials.json (owned by Claude CLI)
    let active_path = claude.join(".credentials.json");
    if active_path.exists() {
        let info = read_credential_info(&active_path);
        profiles.push(CredentialProfile {
            name: active_name.clone(),
            is_active: true,
            info,
            oauth_account: active_oauth,
        });
    }

    // Saved profiles: scan ~/.agent-switch-tools/claude/profiles/ for subdirectories
    if let Ok(entries) = std::fs::read_dir(&profs_dir) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.is_empty() || name == active_name {
                continue;
            }
            let cred_path = entry.path().join("credentials.json");
            if !cred_path.exists() {
                continue;
            }
            let info = read_credential_info(&cred_path);
            let oauth = read_saved_oauth(&profs_dir, &name);
            profiles.push(CredentialProfile {
                name,
                is_active: false,
                info,
                oauth_account: oauth,
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

/// Save current active credentials using email from oauthAccount (auto-detect)
#[tauri::command]
pub async fn save_current_as_profile(app: tauri::AppHandle) -> Result<String, String> {
    let home = home_dir(&app)?;
    let claude = claude_dir(&app)?;
    let claude_data = claude_data_dir(&app)?;
    let profs_dir = profiles_dir(&app)?;
    let active_path = claude.join(".credentials.json");

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

    // Copy credentials into profile dir (overwrite allowed — refreshes tokens)
    let prof_dir = profile_dir(&profs_dir, &email)?;
    let target_path = prof_dir.join("credentials.json");
    std::fs::copy(&active_path, &target_path).map_err(|e| e.to_string())?;
    set_file_600(&target_path);

    // Save oauthAccount as profiles/{email}/oauth.json
    write_saved_oauth(&profs_dir, &email, &oauth)?;

    // Update metadata
    let mut meta = read_meta(&claude_data);
    meta.active_profile_name = Some(email.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&claude_data, &meta)?;

    crate::tray::refresh_tray_menu(&app);
    Ok(email)
}

/// Switch to a different profile
/// 1. Backup current credentials + oauthAccount into their profile dir
/// 2. Copy target credentials → ~/.claude/.credentials.json (keep target file)
/// 3. Restore target's oauthAccount into ~/.claude.json
/// 4. Update metadata
#[tauri::command]
pub async fn switch_credential_profile(
    app: tauri::AppHandle,
    target_name: String,
) -> Result<SwitchResult, String> {
    let home = home_dir(&app)?;
    let claude = claude_dir(&app)?;
    let claude_data = claude_data_dir(&app)?;
    let profs_dir = profiles_dir(&app)?;
    let active_path = claude.join(".credentials.json");
    let target_cred_path = profs_dir.join(&target_name).join("credentials.json");

    if !target_cred_path.exists() {
        return Err(format!("Profile '{}' not found", target_name));
    }

    let target_info = read_credential_info(&target_cred_path);
    let target_was_expired = target_info.is_expired;
    let claude_was_running = check_claude_running();

    // Resolve outgoing profile name
    let mut meta = read_meta(&claude_data);
    let current_email = meta
        .active_profile_name
        .clone()
        .or_else(|| {
            read_oauth_from_claude_json(&home).and_then(|o| o.email_address)
        })
        .unwrap_or_default();

    // Backup outgoing credentials + oauthAccount into their profile dir
    if active_path.exists() && !current_email.is_empty() {
        let prof_dir = profile_dir(&profs_dir, &current_email)?;
        let backup_path = prof_dir.join("credentials.json");
        std::fs::copy(&active_path, &backup_path)
            .map_err(|e| format!("Failed to backup current credentials: {}", e))?;

        if let Some(oauth) = read_oauth_from_claude_json(&home) {
            let _ = write_saved_oauth(&profs_dir, &current_email, &oauth);
        }
    }

    // Activate target: copy into ~/.claude/.credentials.json (keep source file intact)
    std::fs::copy(&target_cred_path, &active_path)
        .map_err(|e| format!("Failed to activate target credentials: {}", e))?;

    // Restore target's oauthAccount into ~/.claude.json
    if let Some(target_oauth) = read_saved_oauth(&profs_dir, &target_name) {
        let _ = update_claude_json_oauth(&home, &target_oauth);
    }

    // Update metadata with usage tracking
    if !current_email.is_empty() {
        record_switch_usage(&mut meta, &current_email);
    }
    meta.active_profile_name = Some(target_name.clone());
    write_meta(&claude_data, &meta)?;

    let message = if claude_was_running {
        "Switched credentials. Restart Claude Code to use new account.".to_string()
    } else if target_was_expired {
        "Switched to expired credentials. Token may auto-refresh on next use.".to_string()
    } else {
        format!("Switched to '{}'.", target_name)
    };

    crate::tray::refresh_tray_menu(&app);
    Ok(SwitchResult {
        success: true,
        claude_was_running,
        target_was_expired,
        message,
    })
}

/// Rename a saved profile directory
#[tauri::command]
pub async fn rename_credential_profile(
    app: tauri::AppHandle,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let profs_dir = profiles_dir(&app)?;
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

/// Delete a saved profile directory (credentials + oauth + any future files)
#[tauri::command]
pub async fn delete_credential_profile(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let profs_dir = profiles_dir(&app)?;
    let prof_dir = profs_dir.join(&name);

    if !prof_dir.exists() {
        return Err(format!("Profile '{}' not found", name));
    }

    std::fs::remove_dir_all(&prof_dir).map_err(|e| e.to_string())?;
    crate::tray::refresh_tray_menu(&app);
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
