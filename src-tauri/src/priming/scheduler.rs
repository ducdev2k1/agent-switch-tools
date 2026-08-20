use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::time::interval;

use crate::modules::providers::claude_cli::config;
use crate::modules::providers::claude_cli::oauth::ensure_fresh_blob;
use crate::modules::shared::active_store::ActiveStore;
use crate::modules::shared::paths::{claude_data_dir, claude_dir, profiles_dir};
use crate::priming::{prime::{prime_account, prime_with_token}, store, PrimeResult};

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
        if !has_credential(app, &name) {
            // A profile whose credential was removed stays in the settings file so its
            // schedule survives re-adding the account. Logging a skip for it every tick
            // would bury the real history under thousands of identical lines.
            continue;
        }

        let result = run_one(app, &name).await;
        store::record_result(app, &name, &today, &result);
        let _ = app.emit("auto-prime-updated", &name);
    }
}

/// True when the profile still has a credential to prime — either it is the active
/// account (whose credential may live in the OS keystore) or it has a saved file.
fn has_credential(app: &AppHandle, name: &str) -> bool {
    let is_active = claude_data_dir(app)
        .ok()
        .map(|dir| config::read_meta(&dir).active_profile_name.as_deref() == Some(name))
        .unwrap_or(false);
    if is_active {
        return claude_dir(app)
            .map(|dir| ActiveStore::new(dir).active_exists())
            .unwrap_or(false);
    }
    profiles_dir(app)
        .map(|d| d.join(name).join("credentials.json").exists())
        .unwrap_or(false)
}

/// Prime a single profile by name (also used by the manual `prime_now` command).
pub async fn run_one(app: &AppHandle, name: &str) -> PrimeResult {
    let not_found = || PrimeResult::Skipped {
        reason: "credentials not found".to_string(),
    };

    // The active account's credential may live in the macOS Keychain (no file), so prime it from
    // the resolved token rather than a path.
    let is_active = claude_data_dir(app)
        .ok()
        .map(|dir| config::read_meta(&dir).active_profile_name.as_deref() == Some(name))
        .unwrap_or(false);

    if is_active {
        let Ok(cl_dir) = claude_dir(app) else { return not_found() };
        let store = ActiveStore::new(cl_dir);
        let Some(blob) = store.read_active() else { return not_found() };
        return match ensure_fresh_blob(&blob).await {
            Ok((token, new_blob)) => {
                if let Some(nb) = new_blob {
                    let _ = store.write_active(&nb);
                }
                prime_with_token(&token).await
            }
            Err(e) => PrimeResult::Failed { reason: format!("token: {e}") },
        };
    }

    match profiles_dir(app).ok().map(|d| d.join(name).join("credentials.json")) {
        Some(path) if path.exists() => prime_account(&path).await,
        _ => not_found(),
    }
}
