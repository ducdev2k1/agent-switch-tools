use serde::Serialize;

use super::config_commands::{CredentialProfile, list_credential_profiles};
use super::quota_commands::{fetch_usage_with_token, read_token_from_creds};

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

/// Build the full webhook payload with profiles + usage data
async fn build_payload(app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
    let profiles = list_credential_profiles(app.clone()).await?;
    let mut profile_entries = Vec::new();

    for profile in &profiles {
        // Try to fetch usage for this profile
        let usage = get_profile_usage_data(app, profile).await;

        profile_entries.push(serde_json::json!({
            "name": profile.name,
            "email": profile.oauth_account.as_ref().and_then(|o| o.email_address.as_ref()),
            "subscription_type": profile.info.subscription_type,
            "rate_limit_tier": profile.info.rate_limit_tier,
            "is_active": profile.is_active,
            "is_expired": profile.info.is_expired,
            "usage": usage,
        }));
    }

    let version = env!("CARGO_PKG_VERSION");
    Ok(serde_json::json!({
        "event": "usage_report",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "app_version": version,
        "data": {
            "profiles": profile_entries,
        }
    }))
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
        home.join(".claude")
            .join(".claude-tools")
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

/// Send webhook: builds payload from Rust (profiles + usage) and POSTs to URL.
/// Set `test_mode = true` to send a lightweight test ping instead.
#[tauri::command]
pub async fn send_webhook(
    app: tauri::AppHandle,
    url: String,
    secret: Option<String>,
    test_mode: Option<bool>,
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
        serde_json::json!({
            "event": "test",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "app_version": env!("CARGO_PKG_VERSION"),
        })
    } else {
        build_payload(&app).await?
    };

    // Send HTTP POST
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json");

    if let Some(ref s) = secret {
        if !s.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", s));
        }
    }

    let res = match req.json(&payload).send().await {
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
