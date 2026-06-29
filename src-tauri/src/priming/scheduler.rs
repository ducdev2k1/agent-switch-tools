use std::path::PathBuf;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::time::interval;

use crate::modules::providers::claude_cli::config;
use crate::modules::shared::paths::{claude_data_dir, claude_dir, profiles_dir};
use crate::priming::{prime::prime_account, store, PrimeResult};

const INITIAL_DELAY: u64 = 15;
const TICK_SECS: u64 = 60;

/// Spawn the priming scheduler. It only runs while the app is open: every minute
/// it primes any enabled profile whose scheduled time has passed and that has not
/// yet primed today.
pub fn spawn_prime_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(INITIAL_DELAY)).await;
        let mut ticker = interval(Duration::from_secs(TICK_SECS));
        loop {
            ticker.tick().await;
            run_due(&app).await;
        }
    });
}

async fn run_due(app: &AppHandle) {
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let hhmm = now.format("%H:%M").to_string();

    for (name, setting) in store::load_all(app) {
        if !setting.enabled || setting.time.is_empty() {
            continue;
        }
        if hhmm < setting.time {
            continue; // scheduled time not reached yet today
        }
        if setting.last_primed_date.as_deref() == Some(today.as_str()) {
            continue; // already primed today
        }

        let result = run_one(app, &name).await;
        store::record_result(app, &name, &today, &result);
        let _ = app.emit("auto-prime-updated", &name);
    }
}

/// Prime a single profile by name (also used by the manual `prime_now` command).
pub async fn run_one(app: &AppHandle, name: &str) -> PrimeResult {
    match creds_path_for(app, name) {
        Some(path) => prime_account(&path).await,
        None => PrimeResult::Skipped {
            reason: "credentials not found".to_string(),
        },
    }
}

fn creds_path_for(app: &AppHandle, name: &str) -> Option<PathBuf> {
    let data_dir = claude_data_dir(app).ok()?;
    let meta = config::read_meta(&data_dir);
    if meta.active_profile_name.as_deref() == Some(name) {
        return claude_dir(app).ok().map(|d| d.join(".credentials.json"));
    }
    let path = profiles_dir(app).ok()?.join(name).join("credentials.json");
    path.exists().then_some(path)
}
