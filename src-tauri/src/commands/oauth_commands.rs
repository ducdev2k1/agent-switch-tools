use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// OAuth account info from ~/.claude.json
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAccount {
    pub account_uuid: Option<String>,
    pub email_address: Option<String>,
    pub organization_uuid: Option<String>,
    pub has_extra_usage_enabled: Option<bool>,
    pub billing_type: Option<String>,
    pub account_created_at: Option<String>,
    pub subscription_created_at: Option<String>,
    pub display_name: Option<String>,
    pub organization_role: Option<String>,
    pub workspace_role: Option<serde_json::Value>,
    pub organization_name: Option<String>,
}

/// Read oauthAccount from ~/.claude.json (home directory)
pub fn read_oauth_from_claude_json(home: &PathBuf) -> Option<OAuthAccount> {
    let path = home.join(".claude.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let oauth = v.get("oauthAccount")?;
    serde_json::from_value(oauth.clone()).ok()
}

/// Read saved oauthAccount from ~/.claude-tools/profiles/{name}/oauth.json
pub fn read_saved_oauth(profiles_dir: &PathBuf, name: &str) -> Option<OAuthAccount> {
    let path = profiles_dir.join(name).join("oauth.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Write oauthAccount to ~/.claude-tools/profiles/{name}/oauth.json
pub fn write_saved_oauth(
    profiles_dir: &PathBuf,
    name: &str,
    account: &OAuthAccount,
) -> Result<(), String> {
    // Ensure the profile directory exists
    let prof_dir = profiles_dir.join(name);
    std::fs::create_dir_all(&prof_dir).map_err(|e| e.to_string())?;

    let path = prof_dir.join("oauth.json");
    let json = serde_json::to_string_pretty(account).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Update oauthAccount field in ~/.claude.json
pub fn update_claude_json_oauth(home: &PathBuf, account: &OAuthAccount) -> Result<(), String> {
    let path = home.join(".claude.json");
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut v: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let account_value = serde_json::to_value(account).map_err(|e| e.to_string())?;
    v["oauthAccount"] = account_value;
    let json = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Tauri command: get current oauthAccount from ~/.claude.json
#[tauri::command]
pub async fn get_oauth_account(app: tauri::AppHandle) -> Result<Option<OAuthAccount>, String> {
    let home = app
        .path()
        .home_dir()
        .map_err(|e| format!("Cannot resolve home directory: {}", e))?;
    Ok(read_oauth_from_claude_json(&home))
}
