use std::path::PathBuf;
use tauri::Manager;

/// Resolve home directory
pub fn home_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .home_dir()
        .map_err(|e| format!("Cannot resolve home directory: {}", e))
}

/// ~/.claude/ — Claude CLI directory (owns active credentials, not managed by us)
pub fn claude_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    home_dir(app).map(|h| h.join(".claude"))
}

/// ~/.agent-switch-tools/ — branded root for all app-managed data
pub fn claude_tools_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = home_dir(app)?.join(".agent-switch-tools");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// ~/.agent-switch-tools/claude/ — Claude CLI data root (same pattern as IDE subdirs)
pub fn claude_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = claude_tools_dir(app)?.join("claude");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// ~/.agent-switch-tools/claude/profiles/ — root for all saved Claude account profiles
pub fn profiles_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = claude_data_dir(app)?.join("profiles");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// ~/.agent-switch-tools/claude/profiles/{name}/ — single account dir, created if missing
pub fn profile_dir(profiles: &PathBuf, name: &str) -> Result<PathBuf, String> {
    let dir = profiles.join(name);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Resolve OS-specific application support directory for an IDE
pub fn ide_app_dir(app: &tauri::AppHandle, app_dir_name: &str) -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(home_dir(app)?
            .join("Library")
            .join("Application Support")
            .join(app_dir_name))
    }
    #[cfg(target_os = "linux")]
    {
        Ok(home_dir(app)?.join(".config").join(app_dir_name))
    }
    #[cfg(target_os = "windows")]
    {
        Ok(home_dir(app)?
            .join("AppData")
            .join("Roaming")
            .join(app_dir_name))
    }
}
