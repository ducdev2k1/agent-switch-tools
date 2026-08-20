use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Emitter;

use crate::modules::providers::claude_cli::config::{self, CredentialProfile};
use crate::modules::providers::claude_cli::auth;
use crate::modules::providers::claude_cli::reconcile::{reconcile_active_profile, validate_email_as_folder};
use crate::modules::shared::active_store::ActiveStore;
use crate::modules::shared::paths::{claude_data_dir, claude_dir, home_dir, profiles_dir};

#[tauri::command]
pub async fn list_credential_profiles(
    app: tauri::AppHandle,
) -> Result<Vec<CredentialProfile>, String> {
    let home = home_dir(&app)?;
    let claude = claude_dir(&app)?;
    let claude_data = claude_data_dir(&app)?;
    let profs_dir = profiles_dir(&app)?;

    config::migrate_legacy_profiles(&claude, &profs_dir);

    // Sync meta with `~/.claude.json` before reading. If the user logged in outside the app,
    // this saves the new credentials into the new email's folder and updates meta — preserving
    // the previous profile folder untouched.
    let (_, drift_detected) = reconcile_active_profile(&home, &claude, &profs_dir, &claude_data)?;
    if drift_detected {
        let _ = app.emit("claude-profile-drift-detected", ());
    }

    let meta = config::read_meta(&claude_data);

    let mut profiles: Vec<CredentialProfile> = vec![];
    let active_oauth = auth::read_oauth_from_claude_json(&home);

    let active_name = active_oauth.as_ref()
        .and_then(|o| o.email_address.clone())
        .or_else(|| meta.active_profile_name.clone())
        .unwrap_or_else(|| "Active".to_string());

    let store = ActiveStore::new(claude.clone());
    if let Some(active_blob) = store.read_active() {
        let info = config::parse_credential_info(&active_blob);
        profiles.push(CredentialProfile {
            name: active_name.clone(),
            is_active: true,
            info,
            oauth_account: active_oauth,
        });
    }

    if let Ok(entries) = std::fs::read_dir(&profs_dir) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.is_empty() || name == active_name { continue; }
            let cred_path = entry.path().join("credentials.json");
            if !cred_path.exists() { continue; }
            let info = config::read_credential_info(&cred_path);
            let oauth = auth::read_saved_oauth(&profs_dir, &name);
            profiles.push(CredentialProfile {
                name,
                is_active: false,
                info,
                oauth_account: oauth,
            });
        }
    }

    profiles.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(profiles)
}

#[tauri::command]
pub async fn save_current_as_profile(app: tauri::AppHandle) -> Result<String, String> {
    let home = home_dir(&app)?;
    let claude = claude_dir(&app)?;
    let claude_data = claude_data_dir(&app)?;
    let profs_dir = profiles_dir(&app)?;
    let store = ActiveStore::new(claude.clone());

    let Some(active_blob) = store.read_active() else {
        return Err("No active credentials found".to_string());
    };

    let oauth = auth::read_oauth_from_claude_json(&home)
        .ok_or("Cannot read oauthAccount from ~/.claude.json")?;
    let email = oauth.email_address.clone()
        .ok_or("No email address found in oauthAccount")?;
    validate_email_as_folder(&email)?;

    let prof_dir = crate::modules::shared::paths::profile_dir(&profs_dir, &email)?;
    let target_path = prof_dir.join("credentials.json");
    std::fs::write(&target_path, &active_blob).map_err(|e| e.to_string())?;
    config::set_file_600(&target_path);

    auth::write_saved_oauth(&profs_dir, &email, &oauth)?;

    let mut meta = config::read_meta(&claude_data);
    let prev_active = meta.active_profile_name.clone();
    #[cfg(debug_assertions)]
    if prev_active.as_ref() != Some(&email) {
        eprintln!(
            "[save_current] Active profile changed: {:?} -> {}",
            prev_active, email
        );
    }
    let _ = &prev_active;
    meta.active_profile_name = Some(email.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    config::write_meta(&claude_data, &meta)?;

    crate::tray::refresh_tray_menu(&app);
    Ok(email)
}

#[tauri::command]
pub async fn switch_credential_profile(
    app: tauri::AppHandle,
    target_name: String,
) -> Result<config::SwitchResult, String> {
    let home = home_dir(&app)?;
    let claude = claude_dir(&app)?;
    let claude_data = claude_data_dir(&app)?;
    let profs_dir = profiles_dir(&app)?;
    let store = ActiveStore::new(claude.clone());
    let target_cred_path = profs_dir.join(&target_name).join("credentials.json");

    if !target_cred_path.exists() {
        return Err(format!("Profile '{}' not found", target_name));
    }

    let target_info = config::read_credential_info(&target_cred_path);
    let target_was_expired = target_info.is_expired;
    let claude_was_running = config::check_claude_running();

    // Reconcile first — if user logged in outside the app, this saves current credentials
    // into the correct folder (matching the actual email) BEFORE we overwrite active_path.
    let (current_email_opt, drift_detected) =
        reconcile_active_profile(&home, &claude, &profs_dir, &claude_data)?;
    if drift_detected {
        let _ = app.emit("claude-profile-drift-detected", ());
    }

    let mut meta = config::read_meta(&claude_data);
    let current_email = current_email_opt.unwrap_or_default();

    // Backup current credentials into its own folder (no-op if drift already saved them).
    // Required when user switches inside the app — meta is in sync, but we still need to
    // preserve the active credentials before overwriting active_path with the target's.
    if !drift_detected && !current_email.is_empty() && current_email != target_name {
        if let Some(active_blob) = store.read_active() {
            let prof_dir = crate::modules::shared::paths::profile_dir(&profs_dir, &current_email)?;
            let backup_path = prof_dir.join("credentials.json");
            let _ = std::fs::write(&backup_path, &active_blob);
            config::set_file_600(&backup_path);

            if let Some(oauth) = auth::read_oauth_from_claude_json(&home) {
                let _ = auth::write_saved_oauth(&profs_dir, &current_email, &oauth);
            }
        }
    }

    let target_blob = std::fs::read_to_string(&target_cred_path).map_err(|e| e.to_string())?;
    store.write_active(&target_blob)?;

    // Always rewrite oauthAccount to the target's identity. If it kept the previous
    // account's email, the next reconcile would treat it as an external login and
    // snapshot the target's credentials into the previous account's folder.
    let target_oauth = auth::read_saved_oauth(&profs_dir, &target_name).unwrap_or_else(|| {
        auth::OAuthAccount {
            email_address: Some(target_name.clone()),
            ..Default::default()
        }
    });
    let _ = auth::update_claude_json_oauth(&home, &target_oauth);

    if !current_email.is_empty() {
        config::record_switch_usage(&mut meta, &current_email);
    }
    meta.active_profile_name = Some(target_name.clone());
    config::write_meta(&claude_data, &meta)?;

    let message = if claude_was_running {
        "Switched credentials. A running session bills the new account right away; the account it displays updates on the next session.".to_string()
    } else if target_was_expired {
        "Switched to expired credentials. Token may auto-refresh on next use.".to_string()
    } else {
        format!("Switched to '{}'.", target_name)
    };

    crate::tray::refresh_tray_menu(&app);
    Ok(config::SwitchResult {
        success: true,
        claude_was_running,
        target_was_expired,
        message,
    })
}

#[tauri::command]
pub async fn rename_credential_profile(
    app: tauri::AppHandle,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let profs_dir = profiles_dir(&app)?;
    let old_dir = profs_dir.join(&old_name);
    let new_dir = profs_dir.join(&new_name);

    if !old_dir.exists() { return Err(format!("Profile '{}' not found", old_name)); }
    if new_dir.exists() { return Err(format!("Profile '{}' already exists", new_name)); }

    std::fs::rename(&old_dir, &new_dir).map_err(|e| e.to_string())?;
    crate::tray::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub async fn delete_credential_profile(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let profs_dir = profiles_dir(&app)?;
    let prof_dir = profs_dir.join(&name);

    if !prof_dir.exists() { return Err(format!("Profile '{}' not found", name)); }
    std::fs::remove_dir_all(&prof_dir).map_err(|e| e.to_string())?;
    crate::tray::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub async fn is_claude_running() -> Result<bool, String> {
    Ok(config::check_claude_running())
}

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
    let active_keys = if env_file_exists { read_env_key_names(&env_path) } else { vec![] };

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
    std::fs::read_to_string(path).unwrap_or_default().lines().count()
}

fn read_env_key_names(env_path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(env_path).unwrap_or_default()
        .lines().filter(|l| !l.starts_with('#') && l.contains('='))
        .map(|l| l.split('=').next().unwrap_or("").trim().to_string())
        .filter(|k| !k.is_empty())
        .collect()
}
