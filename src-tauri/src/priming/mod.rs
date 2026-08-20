use serde::{Deserialize, Serialize};

pub mod prime;
pub mod scheduler;
pub mod store;

/// Per-profile scheduled-priming configuration, persisted to auto-prime.json.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoPrimeSetting {
    pub enabled: bool,
    /// Local time of day to prime, "HH:MM" (24h).
    pub time: String,
    /// Last date a prime ran for this profile, "YYYY-MM-DD" — the once-per-day guard.
    pub last_primed_date: Option<String>,
    pub last_result: Option<String>,
}

/// Outcome of a single prime attempt. `status` is the serde tag the UI switches on.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum PrimeResult {
    /// A new 5h window was opened; carries its reset timestamp (ISO).
    Success { reset_at: String },
    /// A window is already running; carries that window's reset timestamp (ISO).
    Hold { reset_at: String },
    /// Prime attempted but could not be confirmed / failed.
    Failed { reason: String },
    /// Prime skipped before sending (e.g. missing credentials).
    Skipped { reason: String },
}

impl PrimeResult {
    /// Short keyword recorded in settings and the activity log.
    pub fn keyword(&self) -> &'static str {
        match self {
            PrimeResult::Success { .. } => "success",
            PrimeResult::Hold { .. } => "hold",
            PrimeResult::Failed { .. } => "failed",
            PrimeResult::Skipped { .. } => "skip",
        }
    }
}

/// Per-day aggregate of prime outcomes for the stats table.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayStat {
    pub date: String,
    pub success: u32,
    pub failed: u32,
    pub hold: u32,
    pub skip: u32,
}

/// One parsed activity-log line for the priming table.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimeLogEntry {
    pub timestamp: String,
    pub profile: String,
    /// One of the `PrimeResult::keyword()` values.
    pub result: String,
    pub detail: String,
}

/// A page of the priming activity log, newest first, plus the full line count.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimeLogPage {
    pub rows: Vec<PrimeLogEntry>,
    pub total: usize,
}
