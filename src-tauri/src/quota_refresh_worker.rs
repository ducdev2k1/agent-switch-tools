use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration};

use crate::commands::quota_commands::get_usage_limits;

const REFRESH_INTERVAL: u64 = 300; // 5 minutes
const INITIAL_DELAY: u64 = 10; // 10 seconds

/// Spawns a background worker that refreshes quota usage every 5 minutes
/// and emits a `usage-updated` event so the frontend can update without polling.
pub fn spawn_quota_worker(app: AppHandle) {
    tokio::spawn(async move {
        // Wait 10 seconds after startup before first fetch (let app fully initialize)
        tokio::time::sleep(Duration::from_secs(INITIAL_DELAY)).await;

        let mut ticker = interval(Duration::from_secs(REFRESH_INTERVAL)); // 5 minutes
        loop {
            ticker.tick().await;
            match get_usage_limits(app.clone(), None).await {
                Ok(Some(limits)) => {
                    let _ = app.emit("usage-updated", limits);
                }
                Ok(None) => {} // No token available, skip silently
                Err(e) => {
                    eprintln!("[quota_worker] fetch error: {}", e);
                }
            }
        }
    });
}
