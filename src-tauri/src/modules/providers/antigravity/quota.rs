use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::modules::quota::{UsageBucket, UsageLimits};

// 3-tier fallback: sandbox first (per Antigravity-Manager reference) then daily then prod
const QUOTA_API_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:fetchAvailableModels",
    "https://daily-cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
    "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
];

const LOAD_CODE_ASSIST_URL: &str =
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:loadCodeAssist";

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
struct LoadCodeAssistResponse {
    #[serde(rename = "cloudaicompanionProject")]
    project_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuotaResponse {
    models: HashMap<String, ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelInfo {
    #[serde(rename = "quotaInfo")]
    quota_info: Option<QuotaInfo>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuotaInfo {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

/// Step 1 — resolve cloudaicompanionProject via loadCodeAssist.
async fn fetch_project_id(token: &str) -> Option<String> {
    let client = &crate::modules::shared::http::CLIENT;
    let body = serde_json::json!({ "metadata": { "ideType": "ANTIGRAVITY" } });
    let res = client
        .post(LOAD_CODE_ASSIST_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("User-Agent", ANTIGRAVITY_UA)
        .json(&body)
        .send()
        .await
        .ok()?;
    if !res.status().is_success() {
        return None;
    }
    let parsed: LoadCodeAssistResponse = res.json().await.ok()?;
    parsed.project_id
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
    let project_id = fetch_project_id(token).await?;
    let payload = serde_json::json!({ "project": project_id });
    let client = &crate::modules::shared::http::CLIENT;

    for url in QUOTA_API_ENDPOINTS {
        let res = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("User-Agent", ANTIGRAVITY_UA)
            .json(&payload)
            .send()
            .await;

        let Ok(response) = res else { continue };
        if !response.status().is_success() {
            continue;
        }
        let Ok(parsed) = response.json::<QuotaResponse>().await else { continue };
        return Some(map_to_usage_limits(parsed));
    }
    None
}

/// Map Antigravity models into 3 grouped buckets matching native IDE rate-limit pools:
///   1. "Gemini Pro"    — High + Low variants (shared quota)
///   2. "Gemini Flash"  — Flash family (shared quota)
///   3. "Claude / GPT"  — premium pool (Claude Sonnet/Opus + GPT-OSS share the same resetTime)
///
/// All variants within a group share the same remainingFraction/resetTime per API observation,
/// so we collapse them into one row labeled after the shared family name.
///
/// `utilization` carries **remaining %** (0-100) per Antigravity convention:
/// 100 = quota full (bar full, green), 0 = exhausted (bar empty, red).
/// Opposite of Claude CLI. Bucket sets `remaining_based = true` so frontend inverts color.
fn map_to_usage_limits(data: QuotaResponse) -> UsageLimits {
    let mut pro: Option<(f64, String)> = None;
    let mut flash: Option<(f64, String)> = None;
    let mut premium: Option<(f64, String)> = None;

    for (model_id, info) in data.models {
        let Some(q) = info.quota_info else { continue };
        let Some(frac) = q.remaining_fraction else { continue };
        let Some(reset) = q.reset_time else { continue };
        let Some(display) = info.display_name else { continue };
        if display.is_empty() || model_id.starts_with("chat_") || model_id.starts_with("tab_") {
            continue;
        }
        let slot = classify_group(&display);
        // Within a group, pick the entry with lowest remaining (worst-case visibility)
        let target = match slot {
            Group::Pro => &mut pro,
            Group::Flash => &mut flash,
            Group::Premium => &mut premium,
            Group::Other => continue,
        };
        if target.as_ref().map_or(true, |(f, _)| frac < *f) {
            *target = Some((frac, reset));
        }
    }

    let mut buckets: Vec<UsageBucket> = Vec::new();
    if let Some((f, r)) = pro {
        buckets.push(bucket(f, r, "Gemini Pro"));
    }
    if let Some((f, r)) = flash {
        buckets.push(bucket(f, r, "Gemini Flash"));
    }
    if let Some((f, r)) = premium {
        buckets.push(bucket(f, r, "Claude / GPT"));
    }

    UsageLimits {
        buckets,
        ..Default::default()
    }
}

fn bucket(remaining: f64, reset_at: String, label: &str) -> UsageBucket {
    let remaining_pct = (remaining.max(0.0).min(1.0)) * 100.0;
    UsageBucket {
        utilization: Some(remaining_pct),
        resets_at: Some(reset_at),
        label: Some(label.to_string()),
        remaining_based: true,
    }
}

enum Group {
    Pro,
    Flash,
    Premium,
    Other,
}

fn classify_group(display: &str) -> Group {
    let lower = display.to_lowercase();
    if lower.contains("gemini") && lower.contains("pro") {
        Group::Pro
    } else if lower.contains("gemini") && lower.contains("flash") {
        Group::Flash
    } else if lower.contains("claude") || lower.contains("gpt") {
        Group::Premium
    } else {
        Group::Other
    }
}
