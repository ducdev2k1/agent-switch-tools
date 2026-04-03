use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::path_helpers::claude_tools_dir;

/// Manager metadata file: ~/.claude-tools/meta.json
/// Tracks which saved profile is currently active and usage history
const META_FILENAME: &str = "meta.json";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUsage {
    /// ISO 8601 timestamp of last activation
    pub last_active_at: Option<String>,
    /// Total minutes this profile was active
    pub total_active_minutes: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManagerMeta {
    /// Name of the saved profile currently active (e.g., "Work", "Personal")
    pub active_profile_name: Option<String>,
    /// ISO 8601 timestamp of last switch
    pub last_switched_at: Option<String>,
    /// Usage tracking per profile name
    #[serde(default)]
    pub usage_history: std::collections::HashMap<String, ProfileUsage>,
}

impl Default for ManagerMeta {
    fn default() -> Self {
        Self {
            active_profile_name: None,
            last_switched_at: None,
            usage_history: std::collections::HashMap::new(),
        }
    }
}

/// Read metadata from disk, returns default if file missing/corrupt
pub fn read_meta(dir: &PathBuf) -> ManagerMeta {
    let path = dir.join(META_FILENAME);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

/// Write metadata to disk
pub fn write_meta(dir: &PathBuf, meta: &ManagerMeta) -> Result<(), String> {
    let path = dir.join(META_FILENAME);
    let json = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;

    // Set file permissions 600 on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Record usage: calculate how long outgoing profile was active, update history
pub fn record_switch_usage(meta: &mut ManagerMeta, outgoing_name: &str) {
    let now = chrono::Utc::now();

    // Calculate active duration for outgoing profile
    if let Some(last_switched) = &meta.last_switched_at {
        if let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(last_switched) {
            let minutes = (now - last_dt.with_timezone(&chrono::Utc))
                .num_seconds() as f64
                / 60.0;
            let entry = meta
                .usage_history
                .entry(outgoing_name.to_string())
                .or_default();
            entry.total_active_minutes += minutes;
        }
    }

    // Update last_active_at for outgoing profile
    let entry = meta
        .usage_history
        .entry(outgoing_name.to_string())
        .or_default();
    entry.last_active_at = Some(now.to_rfc3339());

    meta.last_switched_at = Some(now.to_rfc3339());
}

// ========== Tauri Commands ==========

#[tauri::command]
pub async fn get_manager_meta(app: tauri::AppHandle) -> Result<ManagerMeta, String> {
    let dir = claude_tools_dir(&app)?;
    Ok(read_meta(&dir))
}

#[tauri::command]
pub async fn set_active_profile_name(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let dir = claude_tools_dir(&app)?;
    let mut meta = read_meta(&dir);
    meta.active_profile_name = Some(name);
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    write_meta(&dir, &meta)
}
