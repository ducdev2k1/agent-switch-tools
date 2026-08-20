use std::collections::BTreeMap;
use std::path::PathBuf;

use tauri::AppHandle;

use crate::modules::shared::activity_log;
use crate::modules::shared::paths::claude_data_dir;
use crate::priming::{AutoPrimeSetting, DayStat, PrimeLogEntry, PrimeLogPage, PrimeResult};

/// Fields per log line: timestamp | profile | result | detail.
/// `detail` is free text that may itself contain " | ", so the split stops here.
const LOG_FIELDS: usize = 4;

type Settings = BTreeMap<String, AutoPrimeSetting>;

fn state_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(claude_data_dir(app)?.join("auto-prime.json"))
}

fn log_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(claude_data_dir(app)?.join("auto-prime.log"))
}

/// Read all per-profile settings (empty map when no file yet).
pub fn load_all(app: &AppHandle) -> Settings {
    let Ok(path) = state_path(app) else {
        return Settings::new();
    };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_all(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = state_path(app)?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

/// Enable/disable and set the daily time for one profile, preserving its guards.
pub fn set(app: &AppHandle, name: &str, enabled: bool, time: String) -> Result<(), String> {
    let mut all = load_all(app);
    let entry = all.entry(name.to_string()).or_default();
    entry.enabled = enabled;
    entry.time = time;
    save_all(app, &all)
}

/// Persist the per-day guard + last result and append an activity-log line.
pub fn record_result(app: &AppHandle, name: &str, today: &str, result: &PrimeResult) {
    let mut all = load_all(app);
    let entry = all.entry(name.to_string()).or_default();
    entry.last_result = Some(result.keyword().to_string());
    // Only Success/Hold consume today's slot; failures stay retryable today.
    if matches!(result, PrimeResult::Success { .. } | PrimeResult::Hold { .. }) {
        entry.last_primed_date = Some(today.to_string());
    }
    let _ = save_all(app, &all);
    append_log(app, name, result);
}

fn append_log(app: &AppHandle, name: &str, result: &PrimeResult) {
    let Ok(path) = log_path(app) else {
        return;
    };
    let detail = match result {
        PrimeResult::Success { reset_at } => format!("reset {reset_at}"),
        PrimeResult::Hold { reset_at } => format!("running, reset {reset_at}"),
        PrimeResult::Failed { reason } => reason.clone(),
        PrimeResult::Skipped { reason } => reason.clone(),
    };
    activity_log::append(&path, &[name, result.keyword(), &detail]);
}

/// One page of the activity log, newest first, with the total line count.
pub fn log_page(app: &AppHandle, offset: usize, limit: usize) -> PrimeLogPage {
    let Ok(path) = log_path(app) else {
        return PrimeLogPage { rows: vec![], total: 0 };
    };
    let (lines, total) = activity_log::page(&path, offset, limit);
    let rows = lines
        .iter()
        .filter_map(|line| {
            let parts = activity_log::split_line(line, LOG_FIELDS);
            if parts.len() < LOG_FIELDS {
                return None;
            }
            Some(PrimeLogEntry {
                timestamp: parts[0].clone(),
                profile: parts[1].clone(),
                result: parts[2].clone(),
                detail: parts[3].clone(),
            })
        })
        .collect();
    PrimeLogPage { rows, total }
}

/// Truncate the log if a previous version let it grow past the cap.
pub fn enforce_log_cap(app: &AppHandle) {
    if let Ok(path) = log_path(app) {
        activity_log::enforce_cap(&path);
    }
}

/// Per-day outcome counts derived from the activity log, newest day first.
pub fn stats(app: &AppHandle) -> Vec<DayStat> {
    let Ok(path) = log_path(app) else {
        return vec![];
    };
    let mut by_day: BTreeMap<String, DayStat> = BTreeMap::new();
    for line in activity_log::lines(&path) {
        let parts = activity_log::split_line(&line, LOG_FIELDS);
        if parts.len() < 3 {
            continue;
        }
        let date: String = parts[0].chars().take(10).collect();
        let keyword = parts[2].as_str();
        let day = by_day.entry(date.clone()).or_default();
        day.date = date;
        match keyword {
            "success" => day.success += 1,
            "failed" => day.failed += 1,
            "hold" => day.hold += 1,
            "skip" => day.skip += 1,
            _ => {}
        }
    }
    let mut out: Vec<DayStat> = by_day.into_values().collect();
    out.reverse();
    out
}
