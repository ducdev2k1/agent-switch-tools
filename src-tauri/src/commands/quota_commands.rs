use std::path::{Path, PathBuf};
use crate::modules::providers::claude_cli::quota as claude_quota;
use crate::modules::providers::claude_cli::oauth as claude_oauth;
use crate::modules::providers::antigravity::quota as anti_quota;
use crate::modules::providers::claude_cli::config::{self, UsageStats};
use crate::modules::quota::{UsageLimits};
use crate::modules::shared::paths::{claude_dir, profiles_dir};

#[tauri::command]
pub async fn get_usage_limits(
    app: tauri::AppHandle,
    force_refresh: Option<bool>,
) -> Result<Option<UsageLimits>, String> {
    let cl_dir = claude_dir(&app)?;
    let creds_path = cl_dir.join(".credentials.json");

    let token = match resolve_claude_token(&creds_path).await {
        Some(t) => t,
        None => return Ok(None),
    };

    Ok(claude_quota::fetch_anthropic_usage(&token, force_refresh.unwrap_or(false)).await)
}

#[tauri::command]
pub async fn get_profile_usage(
    app: tauri::AppHandle,
    profile_name: String,
    force_refresh: Option<bool>,
    is_active: Option<bool>,
) -> Result<Option<UsageLimits>, String> {
    let cl_dir = claude_dir(&app)?;
    let pr_dir = profiles_dir(&app)?;

    let active_path = cl_dir.join(".credentials.json");
    let saved_path = pr_dir.join(&profile_name).join("credentials.json");

    let creds_path = if is_active.unwrap_or(false) {
        active_path
    } else if saved_path.exists() {
        saved_path
    } else if active_path.exists() {
        active_path
    } else {
        return Ok(None);
    };

    let token = match resolve_claude_token(&creds_path).await {
        Some(t) => t,
        None => return Ok(None),
    };

    Ok(claude_quota::fetch_anthropic_usage(&token, force_refresh.unwrap_or(false)).await)
}

/// Return a valid access token from a Claude credentials file.
/// Auto-refreshes via Anthropic OAuth if the stored token is near expiry.
async fn resolve_claude_token(creds_path: &Path) -> Option<String> {
    if !creds_path.exists() {
        return None;
    }
    match claude_oauth::ensure_fresh_token(creds_path).await {
        Ok(t) => Some(t),
        Err(_) => read_token_from_creds(&creds_path.to_path_buf()),
    }
}

#[tauri::command]
pub async fn get_usage_stats(app: tauri::AppHandle) -> Result<UsageStats, String> {
    let cl_dir = claude_dir(&app)?;
    Ok(config::get_usage_stats(&cl_dir))
}

#[tauri::command]
pub async fn get_ide_usage(
    app: tauri::AppHandle,
    ide_type: String,
    profile_name: String,
    is_active: bool,
    force_refresh: Option<bool>,
) -> Result<Option<UsageLimits>, String> {
    use crate::modules::core::path_helpers::{ide_db_path, ide_profiles_dir};
    use crate::modules::core::ide_manager::read_saved_auth_keys;
    use crate::modules::providers::IdeType;
    use crate::modules::core::sqlite_auth::read_ide_auth_keys;

    let ide = IdeType::from_str(&ide_type)?;
    let provider = ide.provider();
    let token_key = match provider.token_key() {
        Some(k) => k,
        None => return Ok(None),
    };

    let auth_data = if is_active {
        let db_path = ide_db_path(&app, &ide)?;
        read_ide_auth_keys(&db_path, provider.auth_keys())?
    } else {
        let pr_dir = ide_profiles_dir(&app, &ide)?;
        read_saved_auth_keys(&pr_dir, &profile_name)?
    };

    // Antigravity uses OAuth with refresh flow; other IDEs use normalize_token directly
    if ide == IdeType::Antigravity {
        use crate::modules::providers::antigravity::oauth::get_fresh_access_token;
        let (token, refresh_opt) = match get_fresh_access_token(&auth_data).await {
            Some(t) => t,
            None => return Ok(None),
        };
        // TODO: persist refreshed token back to profile if refresh_opt signals a rotation
        let _ = refresh_opt;
        return Ok(anti_quota::fetch_antigravity_quota(&token).await);
    }

    let token_raw = match auth_data.get(token_key) {
        Some(t) => t,
        None => return Ok(None),
    };
    let token = provider.normalize_token(token_raw);
    Ok(claude_quota::fetch_anthropic_usage(&token, force_refresh.unwrap_or(false)).await)
}

pub fn read_token_from_creds(creds_path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(creds_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("claudeAiOauth")?
        .get("accessToken")?
        .as_str()
        .map(String::from)
}
