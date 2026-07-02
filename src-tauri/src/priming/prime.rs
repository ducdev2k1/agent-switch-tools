use std::path::Path;
use std::time::Duration;

use serde_json::json;

use crate::modules::providers::claude_cli::oauth::ensure_fresh_token;
use crate::modules::providers::claude_cli::quota::fetch_anthropic_usage;
use crate::modules::quota::UsageLimits;
use crate::priming::PrimeResult;

const PRIME_MODEL: &str = "claude-haiku-4-5-20251001";
const SYSTEM_PREAMBLE: &str = "You are Claude Code, Anthropic's official CLI for Claude.";
const CONFIRM_ATTEMPTS: u32 = 8;
const CONFIRM_INTERVAL_SECS: u64 = 10;

/// Open a fresh 5h window for a Claude profile by sending a one-token "hi", then
/// confirm the reset clock actually moved. Reuses the existing OAuth refresh and
/// quota-read paths. Cross-platform: no OS wake/daemon involvement.
pub async fn prime_account(creds_path: &Path) -> PrimeResult {
    // Ensure a usable access token (auto-refresh when near expiry).
    let token = match ensure_fresh_token(creds_path).await {
        Ok(t) => t,
        Err(e) => return PrimeResult::Failed { reason: format!("token: {e}") },
    };
    prime_with_token(&token).await
}

/// Prime using an already-resolved access token. Used for the active account when its credential
/// lives in the macOS Keychain (no file path to hand to `prime_account`).
pub async fn prime_with_token(token: &str) -> PrimeResult {
    // Don't prime into a window that is already running.
    let before = fetch_anthropic_usage(token, true).await;
    let baseline = before
        .as_ref()
        .and_then(|l| reset_at(l))
        .map(|s| s.to_string());
    if let Some(reset) = before.as_ref().and_then(|l| running_reset(l)) {
        return PrimeResult::Hold { reset_at: reset };
    }

    // Send the priming message.
    if let Err(e) = send_hi(token).await {
        return PrimeResult::Failed { reason: e };
    }

    // Confirm the window anchored to a new future reset.
    match confirm_anchored(token, baseline.as_deref()).await {
        Some(reset_at) => PrimeResult::Success { reset_at },
        None => PrimeResult::Failed {
            reason: "sent but window did not anchor in time".to_string(),
        },
    }
}

/// The five-hour bucket reset timestamp, if present.
fn reset_at(limits: &UsageLimits) -> Option<&str> {
    limits
        .five_hour
        .as_ref()
        .and_then(|b| b.resets_at.as_deref())
}

/// Reset timestamp only when it lies in the future (a window is still running).
fn running_reset(limits: &UsageLimits) -> Option<String> {
    let reset = reset_at(limits)?;
    is_future(reset).then(|| reset.to_string())
}

fn is_future(rfc3339: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map(|t| t.with_timezone(&chrono::Utc) > chrono::Utc::now())
        .unwrap_or(false)
}

async fn send_hi(token: &str) -> Result<(), String> {
    let body = json!({
        "model": PRIME_MODEL,
        "max_tokens": 1,
        "system": [{"type": "text", "text": SYSTEM_PREAMBLE}],
        "messages": [{"role": "user", "content": "hi"}],
    });
    let res = crate::modules::shared::http::CLIENT
        .post("https://api.anthropic.com/v1/messages")
        .bearer_auth(token)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "claude-code-20250219,oauth-2025-04-20")
        .header("User-Agent", "claude-cli/2.0.0 (external, sdk-cli)")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("network: {e}"))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {text}"));
    }
    Ok(())
}

/// Poll the quota endpoint until the reset moves to a new future value vs the
/// pre-send baseline (proof a new window opened), or the budget runs out.
async fn confirm_anchored(token: &str, baseline: Option<&str>) -> Option<String> {
    for _ in 0..CONFIRM_ATTEMPTS {
        tokio::time::sleep(Duration::from_secs(CONFIRM_INTERVAL_SECS)).await;
        if let Some(limits) = fetch_anthropic_usage(token, true).await {
            if let Some(reset) = reset_at(&limits) {
                if is_future(reset) && Some(reset) != baseline {
                    return Some(reset.to_string());
                }
            }
        }
    }
    None
}
