use std::path::PathBuf;

use tauri::AppHandle;

use crate::auto_switch::{
    AutoSwitchConfig, AutoSwitchLogEntry, AutoSwitchLogPage, SwitchReason, COOLDOWN_MAX,
    COOLDOWN_MIN, THRESHOLD_MAX, THRESHOLD_MIN,
};
use crate::modules::shared::activity_log;
use crate::modules::shared::paths::claude_data_dir;

/// Fields per log line: timestamp | from | to | utilization | reason.
const LOG_FIELDS: usize = 5;

fn state_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(claude_data_dir(app)?.join("auto-switch.json"))
}

fn log_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(claude_data_dir(app)?.join("auto-switch.log"))
}

/// Current configuration (defaults when the file is missing or unreadable).
pub fn load(app: &AppHandle) -> AutoSwitchConfig {
    let Ok(path) = state_path(app) else {
        return AutoSwitchConfig::default();
    };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

/// Persist as-is. Only used for internal bookkeeping writes, which carry values
/// already loaded from disk; user input goes through `save`.
fn write(app: &AppHandle, config: &AutoSwitchConfig) -> Result<(), String> {
    let path = state_path(app)?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

/// Persist configuration coming from the UI, forcing threshold and cooldown into
/// their supported ranges. The UI validates too, but the stored values drive an
/// unattended background rule, so they are re-checked at the boundary.
pub fn save(app: &AppHandle, config: &AutoSwitchConfig) -> Result<(), String> {
    let mut sanitized = config.clone();
    if !sanitized.threshold.is_finite() {
        sanitized.threshold = AutoSwitchConfig::default().threshold;
    }
    sanitized.threshold = sanitized.threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX);
    sanitized.cooldown_minutes = sanitized.cooldown_minutes.clamp(COOLDOWN_MIN, COOLDOWN_MAX);
    write(app, &sanitized)
}

/// Start the cooldown, clear the exhausted notice, and log the switch.
pub fn record_switch(app: &AppHandle, from: &str, to: &str, utilization: f64) {
    let mut cfg = load(app);
    cfg.last_auto_switch_at = Some(chrono::Local::now().to_rfc3339());
    cfg.all_exhausted_notified = false;
    let _ = write(app, &cfg);
    append_log(app, from, to, utilization, SwitchReason::ThresholdReached);
}

/// Mark the "no profile below the threshold" notice as sent and log it once.
pub fn record_exhausted(app: &AppHandle, profile: &str, utilization: f64) {
    set_exhausted_notified(app, true);
    append_log(app, profile, "-", utilization, SwitchReason::AllExhausted);
}

/// Flip the anti-spam flag for the exhausted notice (no write when unchanged).
pub fn set_exhausted_notified(app: &AppHandle, notified: bool) {
    let mut cfg = load(app);
    if cfg.all_exhausted_notified == notified {
        return;
    }
    cfg.all_exhausted_notified = notified;
    let _ = write(app, &cfg);
}

fn append_log(app: &AppHandle, from: &str, to: &str, utilization: f64, reason: SwitchReason) {
    let Ok(path) = log_path(app) else {
        return;
    };
    let util = format!("{utilization:.1}");
    activity_log::append(&path, &[from, to, &util, reason.keyword()]);
}

/// Truncate the log if a previous version let it grow past the cap.
pub fn enforce_log_cap(app: &AppHandle) {
    if let Ok(path) = log_path(app) {
        activity_log::enforce_cap(&path);
    }
}

/// One page of the activity log, newest first, with the total line count.
/// Malformed lines are skipped.
pub fn history_page(app: &AppHandle, offset: usize, limit: usize) -> AutoSwitchLogPage {
    let Ok(path) = log_path(app) else {
        return AutoSwitchLogPage { rows: vec![], total: 0 };
    };
    let (lines, total) = activity_log::page(&path, offset, limit);
    let rows = lines.iter().filter_map(|l| parse_log_line(l)).collect();
    AutoSwitchLogPage { rows, total }
}

fn parse_log_line(line: &str) -> Option<AutoSwitchLogEntry> {
    let parts = activity_log::split_line(line, LOG_FIELDS);
    if parts.len() < LOG_FIELDS {
        return None;
    }
    Some(AutoSwitchLogEntry {
        timestamp: parts[0].clone(),
        from: parts[1].clone(),
        to: parts[2].clone(),
        utilization: parts[3].parse::<f64>().ok(),
        reason: parts[4].clone(),
    })
}
