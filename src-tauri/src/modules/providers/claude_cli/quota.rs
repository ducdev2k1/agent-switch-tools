use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use crate::modules::quota::{UsageBucket, UsageLimits};

static USAGE_CACHE: std::sync::LazyLock<Mutex<HashMap<u64, (Option<UsageLimits>, Instant)>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

const CACHE_TTL_SECS: u64 = 120;

fn hash_token(token: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    token.hash(&mut hasher);
    hasher.finish()
}

pub async fn fetch_anthropic_usage(token: &str, force_refresh: bool) -> Option<UsageLimits> {
    let key = hash_token(token);

    // Fresh cache hit → return immediately (unless force_refresh)
    if !force_refresh {
        if let Some(fresh) = read_cache(key, CACHE_TTL_SECS) {
            return fresh;
        }
    }

    // Attempt live fetch; on any failure, degrade gracefully to stale cache
    match try_fetch(token).await {
        Some(limits) => {
            if let Ok(mut cache) = USAGE_CACHE.lock() {
                cache.insert(key, (Some(limits.clone()), Instant::now()));
            }
            Some(limits)
        }
        None => {
            // API failed — surface whatever we last had rather than blanking the UI
            read_cache(key, u64::MAX).flatten()
        }
    }
}

/// Returns Some(data) if cache is fresher than `max_age_secs`.
/// Outer Option: cache hit? Inner Option: may hold None if last fetch yielded None.
fn read_cache(key: u64, max_age_secs: u64) -> Option<Option<UsageLimits>> {
    let cache = USAGE_CACHE.lock().ok()?;
    let (data, fetched_at) = cache.get(&key)?;
    if fetched_at.elapsed().as_secs() < max_age_secs {
        Some(data.clone())
    } else {
        None
    }
}

async fn try_fetch(token: &str) -> Option<UsageLimits> {
    let client = &crate::modules::shared::http::CLIENT;
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
    Some(UsageLimits {
        five_hour: parse_bucket(&raw, "five_hour"),
        seven_day: parse_bucket(&raw, "seven_day"),
        seven_day_sonnet: parse_bucket(&raw, "seven_day_sonnet"),
        buckets: Vec::new(),
    })
}

fn parse_bucket(raw: &serde_json::Value, key: &str) -> Option<UsageBucket> {
    let bucket = raw.get(key)?;
    Some(UsageBucket {
        utilization: bucket.get("utilization").and_then(|v| v.as_f64()),
        resets_at: bucket
            .get("resets_at")
            .and_then(|v| v.as_str())
            .map(String::from),
        label: None,
        remaining_based: false,
    })
}
