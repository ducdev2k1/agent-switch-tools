use serde::Serialize;

use crate::modules::providers::claude_cli::oauth as claude_oauth;
use crate::modules::shared::active_store::ActiveStore;
use crate::modules::shared::paths::{claude_dir, profiles_dir};

/// Result returned to frontend after a refresh attempt
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResult {
    pub success: bool,
    pub message: String,
}

/// Refresh the OAuth token for the active account.
///
/// Refreshes directly against Anthropic's OAuth endpoint (no `claude` CLI
/// subprocess, no quota spent), rotating and persisting the credential back to
/// its store (the login Keychain on macOS, the credentials file otherwise).
#[tauri::command]
pub async fn refresh_active_token(app: tauri::AppHandle) -> Result<RefreshResult, String> {
    let store = ActiveStore::new(claude_dir(&app)?);

    let Some(blob) = store.read_active() else {
        return Ok(RefreshResult {
            success: false,
            message: "No active credentials found".to_string(),
        });
    };

    match claude_oauth::force_refresh_blob(&blob).await {
        Ok((_, new_blob)) => match store.write_active(&new_blob) {
            Ok(()) => Ok(RefreshResult {
                success: true,
                message: "Token refreshed".to_string(),
            }),
            Err(e) => Ok(RefreshResult {
                success: false,
                message: e,
            }),
        },
        Err(e) => Ok(RefreshResult {
            success: false,
            message: e,
        }),
    }
}

/// Refresh the OAuth token for a specific saved profile.
///
/// Refreshes the profile's own credentials file in place — no swapping into the
/// active slot, so it cannot race with the background quota worker or corrupt the
/// active account on failure.
#[tauri::command]
pub async fn refresh_profile_token(
    app: tauri::AppHandle,
    profile_name: String,
) -> Result<RefreshResult, String> {
    let creds_path = profiles_dir(&app)?
        .join(&profile_name)
        .join("credentials.json");

    if !creds_path.exists() {
        return Ok(RefreshResult {
            success: false,
            message: format!("Profile '{}' not found", profile_name),
        });
    }

    match claude_oauth::force_refresh_token(&creds_path).await {
        Ok(_) => Ok(RefreshResult {
            success: true,
            message: "Token refreshed".to_string(),
        }),
        Err(e) => Ok(RefreshResult {
            success: false,
            message: e,
        }),
    }
}
