use std::path::PathBuf;
use std::time::Duration;

/// Raw URL of the single-source-of-truth changelog on the default branch.
/// Editing this file and pushing updates the in-app changelog with no rebuild.
const CHANGELOG_URL: &str =
    "https://raw.githubusercontent.com/ducdev2k1/agent-switch-tools/main/changelog.json";

/// Offline cache filename under the app data dir.
const CACHE_FILE: &str = "changelog-cache.json";

/// Fetch the changelog JSON from GitHub, caching the last success to disk so it
/// still works offline. On network/parse failure, falls back to the cached copy;
/// if there is no cache either, returns an error and the frontend uses its
/// bundled snapshot.
#[tauri::command]
pub async fn fetch_changelog(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let cache_path = super::path_helpers::claude_tools_dir(&app).ok().map(|d| d.join(CACHE_FILE));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    match client.get(CHANGELOG_URL).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(json) => {
                write_cache(cache_path.as_ref(), &json);
                Ok(json)
            }
            Err(e) => read_cache(cache_path).ok_or_else(|| format!("Invalid changelog JSON: {e}")),
        },
        _ => read_cache(cache_path)
            .ok_or_else(|| "Could not fetch changelog and no cache is available".to_string()),
    }
}

/// Best-effort write of the latest changelog to the offline cache.
fn write_cache(path: Option<&PathBuf>, json: &serde_json::Value) {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(txt) = serde_json::to_string(json) {
            let _ = std::fs::write(path, txt);
        }
    }
}

/// Read the offline cache, if present and valid.
fn read_cache(path: Option<PathBuf>) -> Option<serde_json::Value> {
    let txt = std::fs::read_to_string(path?).ok()?;
    serde_json::from_str(&txt).ok()
}
