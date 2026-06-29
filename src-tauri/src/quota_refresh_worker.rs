use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration};

use crate::modules::providers::claude_cli::quota as claude_quota;
use crate::modules::quota::UsageLimits;
use crate::modules::shared::paths::{claude_dir, profiles_dir};

const REFRESH_INTERVAL: u64 = 300; // 5 minutes
const INITIAL_DELAY: u64 = 10; // 10 seconds
const INTER_PROFILE_DELAY_MS: u64 = 1000; // 1s between API calls to avoid rate limiting

/// Read OAuth access token from a credentials.json file
use crate::modules::providers::claude_cli::oauth as claude_oauth;

/// Collect (profile_name, creds_path) for active + saved profiles.
/// Token resolution deferred to per-profile fetch so expired tokens auto-refresh.
fn collect_all_profile_paths(app: &AppHandle) -> Vec<(String, std::path::PathBuf)> {
    let mut result: Vec<(String, std::path::PathBuf)> = Vec::new();

    if let Ok(cl_dir) = claude_dir(app) {
        let active_path = cl_dir.join(".credentials.json");
        if active_path.exists() {
            result.push(("active".to_string(), active_path));
        }
    }

    if let Ok(pr_dir) = profiles_dir(app) {
        if let Ok(entries) = std::fs::read_dir(&pr_dir) {
            let mut names: Vec<String> = entries
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            names.sort();

            for name in names {
                let creds_path = pr_dir.join(&name).join("credentials.json");
                if creds_path.exists() {
                    result.push((name, creds_path));
                }
            }
        }
    }

    result
}

/// Spawns a background worker that refreshes quota usage for ALL profiles every 5 minutes
/// and emits events so the frontend can update without polling.
pub fn spawn_quota_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Wait 10 seconds after startup before first fetch (let app fully initialize)
        tokio::time::sleep(Duration::from_secs(INITIAL_DELAY)).await;

        let mut ticker = interval(Duration::from_secs(REFRESH_INTERVAL));
        loop {
            ticker.tick().await;

            let profiles = collect_all_profile_paths(&app);
            if profiles.is_empty() {
                continue;
            }

            let mut all_usage: HashMap<String, UsageLimits> = HashMap::new();
            let mut active_limits: Option<UsageLimits> = None;

            for (idx, (name, creds_path)) in profiles.iter().enumerate() {
                if idx > 0 {
                    tokio::time::sleep(Duration::from_millis(INTER_PROFILE_DELAY_MS)).await;
                }

                // Auto-refresh expired tokens before fetching quota (persists back to file)
                let token = match claude_oauth::ensure_fresh_token(creds_path).await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                match claude_quota::fetch_anthropic_usage(&token, true).await {
                    Some(limits) => {
                        if idx == 0 {
                            active_limits = Some(limits.clone());
                        }
                        all_usage.insert(name.clone(), limits);
                    }
                    None => {}
                }
            }

            // Emit per-profile map for all profile cards
            if !all_usage.is_empty() {
                // Cache for the tray, then rebuild it so menu labels show fresh %.
                crate::modules::quota::store_profile_usage(&all_usage);
                crate::tray::refresh_tray_menu(&app);
                let _ = app.emit("all-profiles-usage-updated", &all_usage);
            }

            // Backward compat: emit active-only event for CLI status bar / useUsageLimits
            if let Some(limits) = active_limits {
                let _ = app.emit("usage-updated", &limits);
            }
        }
    });
}
