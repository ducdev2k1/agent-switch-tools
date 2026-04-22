// OAuth token refresh for Claude CLI credentials.
//
// `credentials.json` stores:
//   - accessToken   (sk-ant-oat01-*, ~8h TTL)
//   - refreshToken  (sk-ant-ort01-*, rotating — Anthropic issues a new one per refresh)
//   - expiresAt     (unix millis)
//
// When accessToken nears expiry, POST to Anthropic's OAuth endpoint with the stored
// refresh_token and the public client_id. Anthropic returns a fresh access_token AND a
// new refresh_token (rotating refresh tokens — old one invalidated). The file must be
// updated atomically with all three fields.
//
// The client_id below is Claude Code CLI's public OAuth app id. Confirmed via reverse
// engineering of the CLI and compatible with any claudeAiOauth credential the CLI issues.

use std::path::Path;

// Public Claude Code CLI OAuth client id (not a secret — embedded in the CLI binary).
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
// Anthropic Cloudflare rejects default reqwest UA with 1010. Match the CLI format.
const CLAUDE_CLI_USER_AGENT: &str = "claude-cli/1.0.0 (external, cli)";

// Refresh if accessToken expires within this many seconds.
const EXPIRY_SKEW_SECS: i64 = 300;

#[derive(serde::Deserialize)]
struct RefreshResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

/// Ensure the credentials file at `creds_path` has a non-expired accessToken.
/// If the stored token is within EXPIRY_SKEW_SECS of expiry, refresh via Anthropic
/// and rewrite the file atomically (preserving non-OAuth fields).
///
/// Returns the valid accessToken (fresh or untouched) on success.
/// Errors out only on hard failures — callers can fall back to whatever apiKey they had.
pub async fn ensure_fresh_token(creds_path: &Path) -> Result<String, String> {
    let content = std::fs::read_to_string(creds_path)
        .map_err(|e| format!("read credentials: {}", e))?;
    let mut root: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("parse credentials: {}", e))?;

    let oauth = root
        .get_mut("claudeAiOauth")
        .ok_or("credentials missing claudeAiOauth")?;

    let access = oauth
        .get("accessToken")
        .and_then(|v| v.as_str())
        .ok_or("missing accessToken")?
        .to_string();
    let refresh = oauth
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .map(String::from);
    let expires_at_ms = oauth.get("expiresAt").and_then(|v| v.as_i64());

    if !needs_refresh(expires_at_ms) {
        return Ok(access);
    }

    let Some(refresh_token) = refresh else {
        // No refresh_token stored → return whatever we have, caller may 401
        return Ok(access);
    };

    match refresh_access_token(&refresh_token).await {
        Ok(new_tokens) => {
            let now_ms = now_millis();
            if let Some(obj) = oauth.as_object_mut() {
                obj.insert("accessToken".into(), serde_json::Value::String(new_tokens.access_token.clone()));
                obj.insert("refreshToken".into(), serde_json::Value::String(new_tokens.refresh_token));
                obj.insert(
                    "expiresAt".into(),
                    serde_json::Value::Number((now_ms + new_tokens.expires_in * 1000).into()),
                );
            }
            if let Err(e) = write_atomic(creds_path, &root) {
                eprintln!("[claude_cli] failed to persist refreshed credentials: {}", e);
                // still return the new token — in-process use still works
            }
            Ok(new_tokens.access_token)
        }
        Err(e) => {
            eprintln!("[claude_cli] token refresh failed: {}", e);
            Ok(access) // best-effort: return stale token
        }
    }
}

fn needs_refresh(expires_at_ms: Option<i64>) -> bool {
    let Some(exp_ms) = expires_at_ms else { return false };
    let now_s = now_millis() / 1000;
    now_s >= (exp_ms / 1000) - EXPIRY_SKEW_SECS
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

async fn refresh_access_token(refresh_token: &str) -> Result<RefreshResponse, String> {
    let client = &crate::modules::shared::http::CLIENT;
    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": CLIENT_ID,
    });
    let res = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("User-Agent", CLAUDE_CLI_USER_AGENT)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("network: {}", e))?;

    let status = res.status();
    if !status.is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, text));
    }
    res.json::<RefreshResponse>()
        .await
        .map_err(|e| format!("parse response: {}", e))
}

fn write_atomic(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}
