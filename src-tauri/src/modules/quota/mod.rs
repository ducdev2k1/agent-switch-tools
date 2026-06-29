use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// ========== Models (Shared across providers) ==========

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageBucket {
    /// Value in 0-100. Interpretation depends on `remaining_based`:
    /// - `false/None` (default, Claude CLI): percent USED. 100 = quota exhausted, red when high.
    /// - `true` (Antigravity): percent REMAINING. 0 = quota exhausted, red when low.
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
    /// Optional label for dynamic bucket display (used by buckets[] in multi-model providers
    /// like Antigravity). Legacy fixed slots (fiveHour/sevenDay/*) leave this None.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub label: Option<String>,
    /// When true, `utilization` carries percent REMAINING (Antigravity convention).
    /// Frontend inverts bar fill + color threshold (red when low).
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub remaining_based: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageLimits {
    // Legacy Claude CLI slots — kept for backward compat
    pub five_hour: Option<UsageBucket>,
    pub seven_day: Option<UsageBucket>,
    pub seven_day_sonnet: Option<UsageBucket>,
    /// Dynamic buckets for providers with arbitrary model groupings (e.g. Antigravity).
    /// When populated, frontend prefers rendering this list over the legacy fixed slots.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub buckets: Vec<UsageBucket>,
}

// ========== Tray quota cache ==========

/// Latest quota keyed by profile name, populated by the background quota worker.
/// Lets the system tray render usage % synchronously without network calls on
/// the menu thread.
static PROFILE_USAGE: LazyLock<Mutex<HashMap<String, UsageLimits>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Store the latest per-profile quota snapshot from the background worker.
pub fn store_profile_usage(all: &HashMap<String, UsageLimits>) {
    if let Ok(mut cache) = PROFILE_USAGE.lock() {
        for (name, limits) in all {
            cache.insert(name.clone(), limits.clone());
        }
    }
}

/// Read the cached quota for a profile name (None if not fetched yet).
pub fn profile_usage(name: &str) -> Option<UsageLimits> {
    PROFILE_USAGE.lock().ok()?.get(name).cloned()
}
