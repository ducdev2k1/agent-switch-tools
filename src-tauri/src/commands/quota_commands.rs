use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use tauri::Manager;

/// In-memory cache: token_hash → (data, fetched_at)
/// Cache for 2 minutes to avoid rate limiting
static USAGE_CACHE: std::sync::LazyLock<Mutex<HashMap<u64, (Option<UsageLimits>, Instant)>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

const CACHE_TTL_SECS: u64 = 120;

/// Usage bucket from Anthropic OAuth usage API
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageBucket {
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
}

/// Usage limits response: 5h session, 7d total, 7d sonnet
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageLimits {
    pub five_hour: Option<UsageBucket>,
    pub seven_day: Option<UsageBucket>,
    pub seven_day_sonnet: Option<UsageBucket>,
}

/// Local usage stats from history.jsonl
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageStats {
    pub total_sessions: usize,
    pub recent_sessions_7d: usize,
    pub current_model: Option<String>,
    pub has_restrictions: bool,
}

/// Simple hash for cache key (avoid storing full token)
fn hash_token(token: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    token.hash(&mut hasher);
    hasher.finish()
}

/// Fetch usage from Anthropic API with caching (2min TTL).
/// Pass `force_refresh = true` to bypass cache (e.g. on manual refresh button click).
pub async fn fetch_usage_with_token(token: &str, force_refresh: bool) -> Option<UsageLimits> {
    let key = hash_token(token);

    // Check cache first (skip if force_refresh)
    if !force_refresh {
        if let Ok(cache) = USAGE_CACHE.lock() {
            if let Some((data, fetched_at)) = cache.get(&key) {
                if fetched_at.elapsed().as_secs() < CACHE_TTL_SECS {
                    return data.clone();
                }
            }
        }
    }

    // Fetch from API
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None;
    }

    let raw: serde_json::Value = res.json().await.ok()?;
    let result = Some(UsageLimits {
        five_hour: parse_bucket(&raw, "five_hour"),
        seven_day: parse_bucket(&raw, "seven_day"),
        seven_day_sonnet: parse_bucket(&raw, "seven_day_sonnet"),
    });

    // Store in cache
    if let Ok(mut cache) = USAGE_CACHE.lock() {
        cache.insert(key, (result.clone(), Instant::now()));
    }

    result
}

/// Parse a usage bucket from raw JSON
fn parse_bucket(raw: &serde_json::Value, key: &str) -> Option<UsageBucket> {
    let bucket = raw.get(key)?;
    Some(UsageBucket {
        utilization: bucket.get("utilization").and_then(|v| v.as_f64()),
        resets_at: bucket
            .get("resets_at")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}

/// Read OAuth access token from credentials file
pub fn read_token_from_creds(creds_path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(creds_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("claudeAiOauth")?
        .get("accessToken")?
        .as_str()
        .map(String::from)
}

/// Get usage limits for the current active account
#[tauri::command]
pub async fn get_usage_limits(
    app: tauri::AppHandle,
    force_refresh: Option<bool>,
) -> Result<Option<UsageLimits>, String> {
    let home = app.path().home_dir().map_err(|e: tauri::Error| e.to_string())?;
    let creds_path = home.join(".claude").join(".credentials.json");

    let token = match read_token_from_creds(&creds_path) {
        Some(t) => t,
        None => return Ok(None),
    };

    Ok(fetch_usage_with_token(&token, force_refresh.unwrap_or(false)).await)
}

/// Get usage limits for a specific saved profile.
/// Pass `force_refresh: true` to bypass the 2-minute cache.
#[tauri::command]
pub async fn get_profile_usage(
    app: tauri::AppHandle,
    profile_name: String,
    force_refresh: Option<bool>,
    is_active: Option<bool>,
) -> Result<Option<UsageLimits>, String> {
    let home = app.path().home_dir().map_err(|e: tauri::Error| e.to_string())?;

    let active_path = home.join(".claude").join(".credentials.json");
    let saved_path = home
        .join(".claude")
        .join(".claude-tools")
        .join("profiles")
        .join(&profile_name)
        .join("credentials.json");

    // Active profile must use live credentials; saved profiles use their own copy
    let creds_path = if is_active.unwrap_or(false) {
        active_path
    } else if saved_path.exists() {
        saved_path
    } else if active_path.exists() {
        active_path
    } else {
        return Ok(None);
    };

    let token = match read_token_from_creds(&creds_path) {
        Some(t) => t,
        None => return Ok(None),
    };

    Ok(fetch_usage_with_token(&token, force_refresh.unwrap_or(false)).await)
}

/// Get local usage stats (sessions, model, restrictions)
#[tauri::command]
pub async fn get_usage_stats(app: tauri::AppHandle) -> Result<UsageStats, String> {
    let home = app.path().home_dir().map_err(|e: tauri::Error| e.to_string())?;
    let claude_dir = home.join(".claude");

    let history_path = claude_dir.join("history.jsonl");
    let settings_path = claude_dir.join("settings.json");
    let policy_path = claude_dir.join("policy-limits.json");

    let (total_sessions, recent_sessions_7d) = parse_history(&history_path);
    let current_model = read_model_from_settings(&settings_path);
    let has_restrictions = check_policy_restrictions(&policy_path);

    Ok(UsageStats {
        total_sessions,
        recent_sessions_7d,
        current_model,
        has_restrictions,
    })
}

/// Parse history.jsonl to count sessions
fn parse_history(path: &PathBuf) -> (usize, usize) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
    let mut total = 0;
    let mut recent = 0;

    for line in content.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            total += 1;
            if let Some(ts) = v.get("timestamp").and_then(|t| t.as_i64()) {
                let ts_secs = ts / 1000;
                if let Some(dt) = chrono::DateTime::from_timestamp(ts_secs, 0) {
                    if dt > cutoff {
                        recent += 1;
                    }
                }
            }
        }
    }

    (total, recent)
}

/// Read model from settings.json
fn read_model_from_settings(path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("model")?.as_str().map(String::from)
}

/// Check policy restrictions
fn check_policy_restrictions(path: &PathBuf) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(restrictions) = v.get("restrictions").and_then(|r| r.as_object()) {
            return restrictions
                .values()
                .any(|v| v.get("allowed").and_then(|a| a.as_bool()) == Some(false));
        }
    }
    false
}
