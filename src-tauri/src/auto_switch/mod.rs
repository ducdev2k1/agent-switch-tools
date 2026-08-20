use serde::{Deserialize, Serialize};

pub mod evaluate;
pub mod store;

/// Accepted range for the switch threshold (five-hour-window utilization percent).
pub(crate) const THRESHOLD_MIN: f64 = 50.0;
pub(crate) const THRESHOLD_MAX: f64 = 99.0;
/// Accepted range for the cooldown between two automatic switches, in minutes.
pub(crate) const COOLDOWN_MIN: u64 = 5;
pub(crate) const COOLDOWN_MAX: u64 = 120;

const DEFAULT_THRESHOLD: f64 = 90.0;
const DEFAULT_COOLDOWN_MINUTES: u64 = 5;

/// Auto-switch rule configuration, persisted to auto-switch.json.
/// `serde(default)` keeps a hand-edited or older file loadable: missing fields
/// fall back to the defaults below instead of discarding the whole file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoSwitchConfig {
    pub enabled: bool,
    /// Utilization percent of the active profile that triggers a switch.
    pub threshold: f64,
    /// Minimum gap between two automatic switches.
    pub cooldown_minutes: u64,
    /// RFC3339 stamp of the last automatic switch. Persisted so the cooldown
    /// survives an app restart.
    pub last_auto_switch_at: Option<String>,
    /// Set once the "no profile below the threshold" notice has been sent, so it
    /// is not repeated on every worker tick. Cleared as soon as a profile drops
    /// back under the threshold.
    pub all_exhausted_notified: bool,
}

// Hand-written instead of derived: a derived Default would yield threshold 0.0
// and cooldown 0, which would make the rule fire on a completely idle profile.
impl Default for AutoSwitchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: DEFAULT_THRESHOLD,
            cooldown_minutes: DEFAULT_COOLDOWN_MINUTES,
            last_auto_switch_at: None,
            all_exhausted_notified: false,
        }
    }
}

/// Why a line was written to the activity log.
#[derive(Debug, Clone, Copy)]
pub enum SwitchReason {
    /// The active profile hit the threshold and a fallback took over.
    ThresholdReached,
    /// Every profile sat above the threshold, so nothing was switched.
    AllExhausted,
}

impl SwitchReason {
    /// Short keyword recorded in the activity log.
    pub fn keyword(&self) -> &'static str {
        match self {
            SwitchReason::ThresholdReached => "switched",
            SwitchReason::AllExhausted => "exhausted",
        }
    }
}

/// One parsed activity-log line, for the history table.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoSwitchLogEntry {
    pub timestamp: String,
    pub from: String,
    pub to: String,
    /// Utilization of the profile being left; `None` when the log line is malformed.
    pub utilization: Option<f64>,
    /// Either "switched" or "exhausted".
    pub reason: String,
}
