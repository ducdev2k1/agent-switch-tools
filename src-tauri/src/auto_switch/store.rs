use std::io::Write;
use std::path::PathBuf;

use tauri::AppHandle;

use crate::auto_switch::{
    AutoSwitchConfig, AutoSwitchLogEntry, SwitchReason, COOLDOWN_MAX, COOLDOWN_MIN, THRESHOLD_MAX,
    THRESHOLD_MIN,
};
use crate::modules::shared::paths::claude_data_dir;

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
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!(
        "{stamp} | {from} | {to} | {utilization:.1} | {}\n",
        reason.keyword()
    );
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

/// Parsed activity log, newest entry first. Malformed lines are skipped.
pub fn read_history(app: &AppHandle) -> Vec<AutoSwitchLogEntry> {
    let text = log_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let mut entries: Vec<AutoSwitchLogEntry> = text.lines().filter_map(parse_log_line).collect();
    entries.reverse();
    entries
}

fn parse_log_line(line: &str) -> Option<AutoSwitchLogEntry> {
    let parts: Vec<&str> = line.split('|').map(|p| p.trim()).collect();
    if parts.len() < 5 {
        return None;
    }
    Some(AutoSwitchLogEntry {
        timestamp: parts[0].to_string(),
        from: parts[1].to_string(),
        to: parts[2].to_string(),
        utilization: parts[3].parse::<f64>().ok(),
        reason: parts[4].to_string(),
    })
}
