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

/// ~/.claude/.claude-tools/ — branded root for all app-managed data, nested inside Claude CLI dir
pub fn claude_tools_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = claude_dir(app)?.join(".claude-tools");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// ~/.claude/.claude-tools/profiles/ — root for all saved account profiles
pub fn profiles_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = claude_tools_dir(app)?.join("profiles");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// ~/.claude/.claude-tools/profiles/{name}/ — single account dir, created if missing
pub fn profile_dir(profiles: &PathBuf, name: &str) -> Result<PathBuf, String> {
    let dir = profiles.join(name);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
