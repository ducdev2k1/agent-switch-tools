use serde::Serialize;
use std::path::PathBuf;
use tauri::Manager;

/// Result returned to frontend after a refresh attempt
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResult {
    pub success: bool,
    pub message: String,
}

/// Check if the token in a credentials file is expired
fn is_token_expired(creds_path: &PathBuf) -> bool {
    let content = match std::fs::read_to_string(creds_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };
    v.get("claudeAiOauth")
        .and_then(|o| o.get("expiresAt"))
        .and_then(|s| s.as_i64())
        .map(|exp| exp < chrono::Utc::now().timestamp_millis())
        .unwrap_or(false)
}

/// Refresh token by invoking `claude -p "hi" --max-turns 1`.
/// This triggers a real API call which activates the CLI's internal
/// refresh interceptor when the token is expired.
fn refresh_via_cli() -> Result<RefreshResult, String> {
    let output = std::process::Command::new("claude")
        .args(["-p", "hi", "--max-turns", "1"])
        .output()
        .map_err(|e| format!("Failed to run claude: {}", e))?;

    if output.status.success() {
        Ok(RefreshResult {
            success: true,
            message: "Token refreshed via CLI".to_string(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!(
            "CLI refresh failed: {} {}",
            stderr.trim(),
            stdout.trim()
        ))
    }
}

// ========== Tauri Commands ==========

/// Refresh token for the active account using Claude CLI
#[tauri::command]
pub async fn refresh_active_token(
    app: tauri::AppHandle,
) -> Result<RefreshResult, String> {
    let home = app.path().home_dir().map_err(|e: tauri::Error| e.to_string())?;
    let creds_path = home.join(".claude").join(".credentials.json");

    if !is_token_expired(&creds_path) {
        return Ok(RefreshResult {
            success: true,
            message: "Token is still valid".to_string(),
        });
    }

    tokio::task::spawn_blocking(refresh_via_cli)
        .await
        .map_err(|e| format!("Task failed: {}", e))?
}

/// Refresh token for a specific saved profile.
/// Swaps saved credentials into active slot, triggers CLI refresh,
/// copies refreshed file back, then restores original active credentials.
#[tauri::command]
pub async fn refresh_profile_token(
    app: tauri::AppHandle,
    profile_name: String,
) -> Result<RefreshResult, String> {
    let home = app.path().home_dir().map_err(|e: tauri::Error| e.to_string())?;
    let claude_dir = home.join(".claude");
    let creds_path = claude_dir
        .join(".claude-tools")
        .join("profiles")
        .join(&profile_name)
        .join("credentials.json");

    if !creds_path.exists() {
        return Err(format!("Profile '{}' not found", profile_name));
    }

    if !is_token_expired(&creds_path) {
        return Ok(RefreshResult {
            success: true,
            message: "Token is still valid".to_string(),
        });
    }

    let claude_dir_clone = claude_dir.clone();
    let creds_path_clone = creds_path.clone();
    tokio::task::spawn_blocking(move || {
        let active_path = claude_dir_clone.join(".credentials.json");

        // Backup current active credentials
        let active_backup = if active_path.exists() {
            std::fs::read_to_string(&active_path).ok()
        } else {
            None
        };

        // Swap: saved profile → active
        std::fs::copy(&creds_path_clone, &active_path)
            .map_err(|e| format!("Failed to swap credentials: {}", e))?;

        // Trigger CLI refresh
        let result = refresh_via_cli();

        // Copy refreshed credentials back to saved profile
        if result.is_ok() && active_path.exists() {
            let _ = std::fs::copy(&active_path, &creds_path_clone);
        }

        // Restore original active credentials
        if let Some(backup) = active_backup {
            let _ = std::fs::write(&active_path, backup);
        }

        result
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
