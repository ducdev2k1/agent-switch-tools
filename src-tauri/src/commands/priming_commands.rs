use std::collections::BTreeMap;

use tauri::AppHandle;

use crate::priming::{scheduler::run_one, store, AutoPrimeSetting, DayStat, PrimeLogPage, PrimeResult};

/// Enable/disable scheduled priming and set the daily time for one profile.
#[tauri::command]
pub fn set_auto_prime(
    app: AppHandle,
    name: String,
    enabled: bool,
    time: String,
) -> Result<(), String> {
    store::set(&app, &name, enabled, time)
}

/// Apply the same time + enabled flag to every named profile.
#[tauri::command]
pub fn set_auto_prime_all(
    app: AppHandle,
    names: Vec<String>,
    enabled: bool,
    time: String,
) -> Result<(), String> {
    for name in names {
        store::set(&app, &name, enabled, time.clone())?;
    }
    Ok(())
}

/// Run a one-shot prime for a profile now and record the outcome.
#[tauri::command]
pub async fn prime_now(app: AppHandle, name: String) -> Result<PrimeResult, String> {
    let result = run_one(&app, &name).await;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    store::record_result(&app, &name, &today, &result);
    Ok(result)
}

/// Current per-profile priming settings (keyed by profile name).
#[tauri::command]
pub fn get_auto_prime_settings(app: AppHandle) -> BTreeMap<String, AutoPrimeSetting> {
    store::load_all(&app)
}

/// One page of the activity log, newest first. Paged so the whole file never crosses
/// the IPC boundary — it used to be shipped as a single string.
#[tauri::command]
pub async fn get_auto_prime_log_page(app: AppHandle, offset: usize, limit: usize) -> PrimeLogPage {
    store::log_page(&app, offset, limit)
}

/// Per-day outcome counts for the stats table.
#[tauri::command]
pub async fn get_auto_prime_stats(app: AppHandle) -> Vec<DayStat> {
    store::stats(&app)
}
