use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sysinfo::System;

use super::path_helpers::claude_tools_dir;

const DEVICE_FILENAME: &str = "device.json";

/// Persistent device identity stored in ~/.claude/.claude-tools/device.json
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// UUID v4, generated once on first launch, never changes
    pub device_id: String,
    /// User-facing name, defaults to hostname, user can rename
    pub device_name: String,
    /// Auto-detected OS hostname, updated each launch
    pub hostname: String,
    /// ISO 8601 timestamp of first creation
    pub created_at: String,
    /// ISO 8601 timestamp of last app launch
    pub last_seen_at: String,
}

/// Read device info from disk, returns None if file missing or corrupt
fn read_device(dir: &PathBuf) -> Option<DeviceInfo> {
    let path = dir.join(DEVICE_FILENAME);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
}

/// Write device info to disk with 0o600 permissions
fn write_device(dir: &PathBuf, info: &DeviceInfo) -> Result<(), String> {
    let path = dir.join(DEVICE_FILENAME);
    let json = serde_json::to_string_pretty(info).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Load existing device info or create new one. Updates hostname + lastSeenAt each call.
pub fn ensure_device_info(dir: &PathBuf) -> Result<DeviceInfo, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());

    let info = match read_device(dir) {
        Some(mut existing) => {
            // Update transient fields each launch
            existing.hostname = hostname;
            existing.last_seen_at = now;
            existing
        }
        None => {
            // First launch: generate new device identity
            DeviceInfo {
                device_id: uuid::Uuid::new_v4().to_string(),
                device_name: hostname.clone(),
                hostname,
                created_at: now.clone(),
                last_seen_at: now,
            }
        }
    };

    write_device(dir, &info)?;
    Ok(info)
}

// ========== Tauri Commands ==========

/// Read-only: return device info to frontend. Returns error if device.json not yet created.
#[tauri::command]
pub async fn get_device_info(app: tauri::AppHandle) -> Result<DeviceInfo, String> {
    let dir = claude_tools_dir(&app)?;
    read_device(&dir).ok_or_else(|| "Device info not found. Restart the app.".to_string())
}

/// Rename the device (user-facing name only, deviceId unchanged). Max 100 chars.
#[tauri::command]
pub async fn rename_device(app: tauri::AppHandle, name: String) -> Result<DeviceInfo, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Device name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Device name must be 100 characters or less".to_string());
    }

    let dir = claude_tools_dir(&app)?;
    let mut info = read_device(&dir)
        .ok_or_else(|| "Device info not found. Restart the app.".to_string())?;
    info.device_name = name;
    write_device(&dir, &info)?;
    Ok(info)
}
