use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

use super::config_commands::{CredentialProfile, list_credential_profiles};
use super::quota_commands::{fetch_usage_with_token, read_token_from_creds};
use super::session_usage_commands;

/// Response returned to the frontend after a webhook call
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebhookResponse {
    pub success: bool,
    pub status_code: Option<u16>,
    pub message: String,
}

/// Validate URL: must be https:// or http://localhost
fn is_valid_webhook_url(url: &str) -> bool {
    if let Ok(parsed) = url::Url::parse(url) {
        if parsed.scheme() == "https" {
            return true;
        }
        if parsed.scheme() == "http" {
            if let Some(host) = parsed.host_str() {
                return host == "localhost" || host == "127.0.0.1";
            }
        }
    }
    false
}

/// Build the full webhook payload with active profile + usage data + session token usage
async fn build_payload(
    app: &tauri::AppHandle,
    include_credentials: bool,
    include_session_usage: bool,
    member_email: Option<String>,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;

    let all_profiles = list_credential_profiles(app.clone()).await?;
    let active_profile = all_profiles.iter().find(|p| p.is_active);
    let home = app
        .path()
        .home_dir()
        .map_err(|e| format!("Cannot resolve home dir: {}", e))?;

    // Build active profile entry with session_usage attached
    let profile_entry = if let Some(profile) = active_profile {
        let usage = get_profile_usage_data(app, profile).await;

        let mut entry = serde_json::json!({
            "name": profile.name,
            "email": profile.oauth_account.as_ref().and_then(|o| o.email_address.as_ref()),
            "subscription_type": profile.info.subscription_type,
            "rate_limit_tier": profile.info.rate_limit_tier,
            "is_active": profile.is_active,
            "is_expired": profile.info.is_expired,
            "usage": usage,
        });

        // Optionally include raw credentials.json content
        if include_credentials {
            let creds_path = home.join(".claude").join(".credentials.json");
            if let Ok(content) = std::fs::read_to_string(&creds_path) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    entry["credentials"] = parsed;
                }
            }
        }

        // Attach session token usage directly to the active profile
        if include_session_usage {
            if let Ok(claude_dir) = super::path_helpers::claude_dir(app) {
                let since = chrono::Utc::now() - chrono::Duration::minutes(10);
                let sessions = session_usage_commands::parse_session_logs(&claude_dir, since);
                let summary = session_usage_commands::build_aggregate(&sessions);

                entry["session_usage"] = serde_json::json!({
                    "summary": summary,
                    "sessions": sessions,
                });
            }
        }

        Some(entry)
    } else {
        None
    };

    let version = env!("CARGO_PKG_VERSION");
    let sys_info = super::system_info_commands::collect_system_info();

    // Load device identity for per-device tracking
    let device_info = super::path_helpers::claude_tools_dir(app)
        .and_then(|dir| super::device_commands::ensure_device_info(&dir))
        .ok();

    let mut payload = serde_json::json!({
        "event": "usage_report",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "app_version": version,
        "system_info": sys_info,
        "data": {
            "active_profile": profile_entry,
        }
    });

    // Inject device_info for per-device usage tracking
    if let Some(ref dev) = device_info {
        payload["device_info"] = serde_json::json!({
            "device_id": dev.device_id,
            "device_name": dev.device_name,
            "hostname": dev.hostname,
        });
    }

    // Include member_email at top level if provided
    if let Some(ref email) = member_email {
        if !email.is_empty() {
            payload["member_email"] = serde_json::json!(email);
        }
    }

    Ok(payload)
}

/// Fetch usage data for a single profile (returns null on failure)
async fn get_profile_usage_data(
    app: &tauri::AppHandle,
    profile: &CredentialProfile,
) -> Option<serde_json::Value> {
    use tauri::Manager;

    let home = app.path().home_dir().ok()?;
    let creds_path = if profile.is_active {
        home.join(".claude").join(".credentials.json")
    } else {
        home.join(".agent-switch-tools")
            .join("claude")
            .join("profiles")
            .join(&profile.name)
            .join("credentials.json")
    };

    let token = read_token_from_creds(&creds_path)?;
    let limits = fetch_usage_with_token(&token, false).await?;

    Some(serde_json::json!({
        "five_hour": limits.five_hour.as_ref().map(|b| serde_json::json!({
            "utilization": b.utilization,
            "resets_at": b.resets_at,
        })),
        "seven_day": limits.seven_day.as_ref().map(|b| serde_json::json!({
            "utilization": b.utilization,
            "resets_at": b.resets_at,
        })),
        "seven_day_sonnet": limits.seven_day_sonnet.as_ref().map(|b| serde_json::json!({
            "utilization": b.utilization,
            "resets_at": b.resets_at,
        })),
    }))
}

/// Send webhook: builds payload from Rust (profiles + usage + session tokens) and POSTs to URL.
/// Set `test_mode = true` to send a lightweight test ping instead.
#[tauri::command]
pub async fn send_webhook(
    app: tauri::AppHandle,
    url: String,
    secret: Option<String>,
    api_key: Option<String>,
    test_mode: Option<bool>,
    include_credentials: Option<bool>,
    include_session_usage: Option<bool>,
    member_email: Option<String>,
) -> Result<WebhookResponse, String> {
    // Validate URL
    if !is_valid_webhook_url(&url) {
        return Ok(WebhookResponse {
            success: false,
            status_code: None,
            message: "Invalid URL: HTTPS required (localhost allowed for testing)".to_string(),
        });
    }

    // Build payload
    let payload = if test_mode.unwrap_or(false) {
        let sys_info = super::system_info_commands::collect_system_info();
        let mut p = serde_json::json!({
            "event": "test",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "app_version": env!("CARGO_PKG_VERSION"),
            "system_info": sys_info,
        });
        if let Some(ref email) = member_email {
            if !email.is_empty() {
                p["member_email"] = serde_json::json!(email);
            }
        }
        // Inject device_info for per-device tracking
        if let Ok(dir) = super::path_helpers::claude_tools_dir(&app) {
            if let Ok(dev) = super::device_commands::ensure_device_info(&dir) {
                p["device_info"] = serde_json::json!({
                    "device_id": dev.device_id,
                    "device_name": dev.device_name,
                    "hostname": dev.hostname,
                });
            }
        }
        p
    } else {
        build_payload(
            &app,
            include_credentials.unwrap_or(false),
            include_session_usage.unwrap_or(true),
            member_email,
        )
        .await?
    };

    // Send HTTP POST
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Serialize body once for HMAC signing
    let body_str = serde_json::to_string(&payload)
        .map_err(|e| format!("Failed to serialize payload: {}", e))?;

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json");

    let has_api_key = api_key.as_ref().map_or(false, |k| !k.is_empty());

    if has_api_key {
        let key = api_key.as_ref().unwrap();
        let timestamp = chrono::Utc::now().timestamp_millis().to_string();

        // HMAC-SHA256(api_key, "{timestamp}.{body}")
        let sign_input = format!("{}.{}", timestamp, body_str);
        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
            .map_err(|e| format!("HMAC key error: {}", e))?;
        mac.update(sign_input.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // Extract device_id from payload for X-Device-Id header
        let device_id = payload
            .get("device_info")
            .and_then(|d| d.get("device_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        req = req
            .header("X-Device-Id", device_id)
            .header("X-Timestamp", &timestamp)
            .header("X-Signature", &signature);
    } else if let Some(ref s) = secret {
        // Fallback: Bearer token auth
        if !s.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", s));
        }
    }

    let res = match req.body(body_str).send().await {
        Ok(r) => r,
        Err(e) => {
            let msg = if e.is_timeout() {
                "Connection timed out (10s)".to_string()
            } else if e.is_connect() {
                "Connection refused".to_string()
            } else {
                format!("Request failed: {}", e)
            };
            return Ok(WebhookResponse {
                success: false,
                status_code: None,
                message: msg,
            });
        }
    };

    let status = res.status();
    let status_code = status.as_u16();

    if status.is_success() {
        Ok(WebhookResponse {
            success: true,
            status_code: Some(status_code),
            message: format!("OK ({})", status_code),
        })
    } else {
        let body = res.text().await.unwrap_or_default();
        let msg = if body.chars().count() > 200 {
            let truncated: String = body.chars().take(200).collect();
            format!("HTTP {} — {}...", status_code, truncated)
        } else if body.is_empty() {
            format!("HTTP {}", status_code)
        } else {
            format!("HTTP {} — {}", status_code, body)
        };
        Ok(WebhookResponse {
            success: false,
            status_code: Some(status_code),
            message: msg,
        })
    }
}
