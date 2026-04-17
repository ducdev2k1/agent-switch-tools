use std::path::PathBuf;
use tauri::Manager;

use super::registry::{IdeInfo, IdeType};
use crate::commands::path_helpers::claude_tools_dir;

/// Resolve the OS-specific application data directory for an IDE
/// Linux:   ~/.config/{AppName}/
/// macOS:   ~/Library/Application Support/{AppName}/
/// Windows: %APPDATA%/{AppName}/
fn ide_app_data_dir(app: &tauri::AppHandle, app_dir_name: &str) -> Result<PathBuf, String> {
    let home = app
        .path()
        .home_dir()
        .map_err(|e| format!("Cannot resolve home directory: {}", e))?;

    #[cfg(target_os = "linux")]
    {
        Ok(home.join(".config").join(app_dir_name))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(home
            .join("Library")
            .join("Application Support")
            .join(app_dir_name))
    }

    #[cfg(target_os = "windows")]
    {
        // Use APPDATA env var, fallback to home/AppData/Roaming
        let appdata = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join("AppData").join("Roaming"));
        Ok(appdata.join(app_dir_name))
    }
}

/// Resolve absolute path to IDE's state.vscdb
/// e.g. ~/.config/Cursor/User/globalStorage/state.vscdb
pub fn ide_db_path(app: &tauri::AppHandle, ide_type: &IdeType) -> Result<PathBuf, String> {
    let config = ide_type.config();
    let app_dir = ide_app_data_dir(app, config.app_dir_name)?;
    Ok(app_dir
        .join("User")
        .join("globalStorage")
        .join("state.vscdb"))
}

/// Check if an IDE is installed by verifying state.vscdb exists
pub fn ide_is_installed(app: &tauri::AppHandle, ide_type: &IdeType) -> bool {
    ide_db_path(app, ide_type)
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// ~/.agent-switch-tools/{ide}/ — per-IDE root for app-managed data
pub fn ide_tools_dir(app: &tauri::AppHandle, ide_type: &IdeType) -> Result<PathBuf, String> {
    let dir = claude_tools_dir(app)?.join(ide_type.id());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// ~/.agent-switch-tools/{ide}/profiles/ — saved profiles for this IDE
pub fn ide_profiles_dir(app: &tauri::AppHandle, ide_type: &IdeType) -> Result<PathBuf, String> {
    let dir = ide_tools_dir(app, ide_type)?.join("profiles");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// List all installed IDEs with their info
#[tauri::command]
pub async fn list_installed_ides(app: tauri::AppHandle) -> Result<Vec<IdeInfo>, String> {
    let mut ides = Vec::new();
    for ide_type in IdeType::all() {
        let config = ide_type.config();
        ides.push(IdeInfo {
            ide_type: *ide_type,
            display_name: config.display_name.to_string(),
            is_installed: ide_is_installed(&app, ide_type),
        });
    }
    Ok(ides)
}
