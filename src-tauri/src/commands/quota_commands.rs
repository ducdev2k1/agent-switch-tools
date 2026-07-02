use std::path::{Path, PathBuf};
use crate::modules::providers::claude_cli::quota as claude_quota;
use crate::modules::providers::claude_cli::oauth as claude_oauth;
use crate::modules::providers::antigravity::quota as anti_quota;
use crate::modules::providers::claude_cli::config::{self, UsageStats};
use crate::modules::quota::{UsageLimits};
use crate::modules::shared::active_store::ActiveStore;
use crate::modules::shared::paths::{claude_dir, profiles_dir};

#[tauri::command]
pub async fn get_usage_limits(
    app: tauri::AppHandle,
    force_refresh: Option<bool>,
) -> Result<Option<UsageLimits>, String> {
    let store = ActiveStore::new(claude_dir(&app)?);

    let token = match resolve_active_token(&store).await {
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

    let saved_path = pr_dir.join(&profile_name).join("credentials.json");

    let token = if is_active.unwrap_or(false) {
        resolve_active_token(&ActiveStore::new(cl_dir)).await
    } else if saved_path.exists() {
        resolve_claude_token(&saved_path).await
    } else {
        // Not saved yet: fall back to the active store (the profile may be the active account).
        resolve_active_token(&ActiveStore::new(cl_dir)).await
    };

    let token = match token {
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

/// Return a valid access token for the active account, reading from its store (macOS Keychain or
/// file). Auto-refreshes near expiry and persists the rotated blob back to the same store.
async fn resolve_active_token(store: &ActiveStore) -> Option<String> {
    let blob = store.read_active()?;
    match claude_oauth::ensure_fresh_blob(&blob).await {
        Ok((token, Some(new_blob))) => {
            let _ = store.write_active(&new_blob);
            Some(token)
        }
        Ok((token, None)) => Some(token),
        Err(_) => parse_token(&blob),
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
    use crate::modules::core::path_helpers::{ide_credential_source, ide_profiles_dir};
    use crate::modules::core::ide_manager::{read_saved_auth_keys, write_saved_auth_keys};
    use crate::modules::providers::IdeType;
    use crate::modules::providers::antigravity::CLI_TOKEN_KEY;

    let ide = IdeType::from_str(&ide_type)?;
    let provider = ide.provider();

    let auth_data = if is_active {
        let source = ide_credential_source(&app, &ide)?;
        source.read(provider.auth_keys())?
    } else {
        let pr_dir = ide_profiles_dir(&app, &ide)?;
        read_saved_auth_keys(&pr_dir, &profile_name)?
    };

    // Antigravity family authenticates via Google OAuth tokens (not a flat apiKey).
    match ide {
        IdeType::Antigravity | IdeType::AntigravityIde => {
            use crate::modules::providers::antigravity::oauth::get_fresh_access_token;
            let (token, _refresh_opt) = match get_fresh_access_token(&auth_data).await {
                Some(t) => t,
                None => return Ok(None),
            };
            // Note: IDE proto token is re-derived per call; CLI variant persists below.
            return Ok(anti_quota::fetch_antigravity_quota(&token).await);
        }
        IdeType::AntigravityCli => {
            use crate::modules::providers::antigravity::oauth::get_fresh_cli_access_token;
            let json = match auth_data.get(CLI_TOKEN_KEY) {
                Some(j) => j,
                None => return Ok(None),
            };
            let (token, updated) = match get_fresh_cli_access_token(json).await {
                Some(t) => t,
                None => return Ok(None),
            };
            // Persist a refreshed CLI token (file or saved profile) so it survives restarts.
            if let Some(updated_json) = updated {
                if is_active {
                    if let Ok(source) = ide_credential_source(&app, &ide) {
                        let mut m = std::collections::HashMap::new();
                        m.insert(CLI_TOKEN_KEY.to_string(), updated_json);
                        let _ = source.write(CLI_TOKEN_KEY, &m);
                    }
                } else if let Ok(pr_dir) = ide_profiles_dir(&app, &ide) {
                    let mut updated_data = auth_data.clone();
                    updated_data.insert(CLI_TOKEN_KEY.to_string(), updated_json);
                    let _ = write_saved_auth_keys(&pr_dir, &profile_name, &updated_data);
                }
            }
            return Ok(anti_quota::fetch_antigravity_quota(&token).await);
        }
        _ => {}
    }

    // Other IDEs (cursor/windsurf): flat token via normalize_token.
    let token_key = match provider.token_key() {
        Some(k) => k,
        None => return Ok(None),
    };
    let token_raw = match auth_data.get(token_key) {
        Some(t) => t,
        None => return Ok(None),
    };
    let token = provider.normalize_token(token_raw);
    Ok(claude_quota::fetch_anthropic_usage(&token, force_refresh.unwrap_or(false)).await)
}

pub fn read_token_from_creds(creds_path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(creds_path).ok()?;
    parse_token(&content)
}

/// Extract the OAuth access token from a credentials JSON blob (file OR macOS Keychain value).
pub fn parse_token(blob: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(blob).ok()?;
    v.get("claudeAiOauth")?
        .get("accessToken")?
        .as_str()
        .map(String::from)
}
