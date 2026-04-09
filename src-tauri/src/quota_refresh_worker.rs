use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration};

use crate::commands::quota_commands::{
    collect_all_profile_tokens, fetch_usage_with_token, UsageLimits,
};

const REFRESH_INTERVAL: u64 = 300; // 5 minutes
const INITIAL_DELAY: u64 = 10; // 10 seconds
const INTER_PROFILE_DELAY_MS: u64 = 1000; // 1s between API calls to avoid rate limiting

/// Spawns a background worker that refreshes quota usage for ALL profiles every 5 minutes
/// and emits events so the frontend can update without polling.
pub fn spawn_quota_worker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Wait 10 seconds after startup before first fetch (let app fully initialize)
        tokio::time::sleep(Duration::from_secs(INITIAL_DELAY)).await;

        let mut ticker = interval(Duration::from_secs(REFRESH_INTERVAL));
        loop {
            ticker.tick().await;

            let profiles = collect_all_profile_tokens(&app);
            if profiles.is_empty() {
                continue;
            }

            let mut all_usage: HashMap<String, UsageLimits> = HashMap::new();
            let mut active_limits: Option<UsageLimits> = None;

            for (idx, (name, token)) in profiles.iter().enumerate() {
                // Small delay between calls to avoid rate limiting (skip first)
                if idx > 0 {
                    tokio::time::sleep(Duration::from_millis(INTER_PROFILE_DELAY_MS)).await;
                }

                match fetch_usage_with_token(token, true).await {
                    Some(limits) => {
                        // First profile is always the active one
                        if idx == 0 {
                            active_limits = Some(limits.clone());
                        }
                        all_usage.insert(name.clone(), limits);
                    }
                    None => {
                        // Token invalid or API error — skip silently
                    }
                }
            }

            // Emit per-profile map for all profile cards
            if !all_usage.is_empty() {
                let _ = app.emit("all-profiles-usage-updated", &all_usage);
            }

            // Backward compat: emit active-only event for CLI status bar / useUsageLimits
            if let Some(limits) = active_limits {
                let _ = app.emit("usage-updated", &limits);
            }
        }
    });
}
