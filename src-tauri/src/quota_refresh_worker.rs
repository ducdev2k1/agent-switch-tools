use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration};

use crate::modules::providers::claude_cli::quota as claude_quota;
use crate::modules::quota::UsageLimits;
use crate::modules::shared::active_store::ActiveStore;
use crate::modules::shared::paths::{claude_dir, profiles_dir};

const REFRESH_INTERVAL: u64 = 300; // 5 minutes
const INITIAL_DELAY: u64 = 10; // 10 seconds
const INTER_PROFILE_DELAY_MS: u64 = 1000; // 1s between API calls to avoid rate limiting

use crate::modules::providers::claude_cli::oauth as claude_oauth;

/// Where a profile's credential lives. The active account may be in the macOS Keychain (no file).
enum CredSource {
    Active,
    File(std::path::PathBuf),
}

/// Collect (profile_name, credential source) for active + saved profiles.
/// Token resolution deferred to per-profile fetch so expired tokens auto-refresh.
fn collect_all_profile_sources(app: &AppHandle) -> Vec<(String, CredSource)> {
    let mut result: Vec<(String, CredSource)> = Vec::new();

    if let Ok(cl_dir) = claude_dir(app) {
        if ActiveStore::new(cl_dir).active_exists() {
            result.push(("active".to_string(), CredSource::Active));
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
                    result.push((name, CredSource::File(creds_path)));
                }
            }
        }
    }

    result
}

/// Resolve a valid access token for a source, auto-refreshing near expiry and persisting the
/// rotated credential back to its store (Keychain/file for active, file for saved profiles).
async fn resolve_source_token(app: &AppHandle, source: &CredSource) -> Option<String> {
    match source {
        CredSource::Active => {
            let store = ActiveStore::new(claude_dir(app).ok()?);
            let blob = store.read_active()?;
            match claude_oauth::ensure_fresh_blob(&blob).await {
                Ok((token, Some(new_blob))) => {
                    let _ = store.write_active(&new_blob);
                    Some(token)
                }
                Ok((token, None)) => Some(token),
                Err(_) => None,
            }
        }
        CredSource::File(path) => claude_oauth::ensure_fresh_token(path).await.ok(),
    }
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

            let profiles = collect_all_profile_sources(&app);
            if profiles.is_empty() {
                continue;
            }

            let mut all_usage: HashMap<String, UsageLimits> = HashMap::new();
            let mut active_limits: Option<UsageLimits> = None;

            for (idx, (name, source)) in profiles.iter().enumerate() {
                if idx > 0 {
                    tokio::time::sleep(Duration::from_millis(INTER_PROFILE_DELAY_MS)).await;
                }

                // Auto-refresh expired tokens before fetching quota (persists back to store)
                let token = match resolve_source_token(&app, source).await {
                    Some(t) => t,
                    None => continue,
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

            // Signal the usage/cost view to refetch its report on the same cadence.
            let _ = app.emit("usage-changed", ());

            // Backward compat: emit active-only event for CLI status bar / useUsageLimits
            if let Some(limits) = active_limits {
                let _ = app.emit("usage-updated", &limits);
            }
        }
    });
}
