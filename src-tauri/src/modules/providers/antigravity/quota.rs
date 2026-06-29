use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::modules::quota::{UsageBucket, UsageLimits};

// Antigravity's current quota model (per Google Gemini plan update) exposes per-group
// Weekly + 5-hour rate-limit buckets via `retrieveUserQuotaSummary`. This replaces the older
// per-model `fetchAvailableModels` remainingFraction (single window). No project id is needed.
// 3-tier host fallback mirrors the native client (prod → sandbox → legacy).
const QUOTA_SUMMARY_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:retrieveUserQuotaSummary",
    "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
];

// Antigravity native UA — Google's quota API gates on this string
const ANTIGRAVITY_UA: &str =
    "Antigravity/4.1.32 (X11; Linux x86_64) Chrome/132.0.6834.160 Electron/39.2.3";

const CACHE_TTL_SECS: u64 = 120;

static QUOTA_CACHE: std::sync::LazyLock<Mutex<HashMap<u64, (Option<UsageLimits>, Instant)>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn hash_token(token: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    token.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Deserialize)]
struct QuotaSummary {
    #[serde(default)]
    groups: Vec<QuotaGroup>,
}

#[derive(Debug, Deserialize)]
struct QuotaGroup {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(default)]
    buckets: Vec<QuotaBucketRaw>,
}

#[derive(Debug, Deserialize)]
struct QuotaBucketRaw {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
}

pub async fn fetch_antigravity_quota(token: &str) -> Option<UsageLimits> {
    let cache_key = hash_token(token);

    // Fresh cache → return immediately
    if let Some(fresh) = read_cache(cache_key, CACHE_TTL_SECS) {
        return fresh;
    }

    // Live fetch; on failure fall back to stale cache so UI keeps showing last-known data
    match try_fetch(token).await {
        Some(limits) => {
            if let Ok(mut cache) = QUOTA_CACHE.lock() {
                cache.insert(cache_key, (Some(limits.clone()), Instant::now()));
            }
            Some(limits)
        }
        None => read_cache(cache_key, u64::MAX).flatten(),
    }
}

fn read_cache(key: u64, max_age_secs: u64) -> Option<Option<UsageLimits>> {
    let cache = QUOTA_CACHE.lock().ok()?;
    let (data, fetched_at) = cache.get(&key)?;
    if fetched_at.elapsed().as_secs() < max_age_secs {
        Some(data.clone())
    } else {
        None
    }
}

async fn try_fetch(token: &str) -> Option<UsageLimits> {
    let client = &crate::modules::shared::http::CLIENT;
    let empty = serde_json::json!({});

    for url in QUOTA_SUMMARY_ENDPOINTS {
        let res = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("User-Agent", ANTIGRAVITY_UA)
            .json(&empty)
            .send()
            .await;

        let Ok(response) = res else { continue };
        if !response.status().is_success() {
            continue;
        }
        let Ok(parsed) = response.json::<QuotaSummary>().await else {
            continue;
        };
        let limits = map_summary(parsed);
        if !limits.buckets.is_empty() {
            return Some(limits);
        }
    }
    None
}

/// Map the quota summary into flat buckets. Each group (e.g. "Gemini Models",
/// "Claude and GPT models") contributes its Weekly + 5-hour buckets, labeled
/// "<group> — <window>". `utilization` carries **remaining %** (Antigravity convention):
/// 100 = full, 0 = exhausted; `remaining_based = true` tells the frontend to invert colors.
fn map_summary(summary: QuotaSummary) -> UsageLimits {
    let mut buckets: Vec<UsageBucket> = Vec::new();

    for group in summary.groups {
        let group_name = group
            .display_name
            .unwrap_or_default()
            .trim_end_matches(" models")
            .trim_end_matches(" Models")
            .to_string();

        for b in group.buckets {
            let Some(frac) = b.remaining_fraction else {
                continue;
            };
            let window = b.display_name.unwrap_or_default();
            let label = if group_name.is_empty() {
                window
            } else if window.is_empty() {
                group_name.clone()
            } else {
                format!("{} — {}", group_name, window)
            };
            buckets.push(UsageBucket {
                utilization: Some((frac.max(0.0).min(1.0)) * 100.0),
                resets_at: b.reset_time,
                label: Some(label),
                remaining_based: true,
            });
        }
    }

    UsageLimits {
        buckets,
        ..Default::default()
    }
}
