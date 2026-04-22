use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAccount {
    pub email_address: Option<String>,
    pub login_type: Option<String>,
    pub expires_at: Option<i64>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
}

pub fn read_oauth_from_claude_json(home_dir: &PathBuf) -> Option<OAuthAccount> {
    let path = home_dir.join(".claude.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let oauth = v.get("claudeAiOauth")?;
    serde_json::from_value(oauth.clone()).ok()
}

pub fn update_claude_json_oauth(home_dir: &PathBuf, account: &OAuthAccount) -> Result<(), String> {
    let path = home_dir.join(".claude.json");
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut v: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    v["claudeAiOauth"] = serde_json::to_value(account).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_saved_oauth(profs_dir: &PathBuf, name: &str) -> Option<OAuthAccount> {
    let path = profs_dir.join(name).join("oauth.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_saved_oauth(profs_dir: &PathBuf, name: &str, account: &OAuthAccount) -> Result<(), String> {
    let prof_dir = profs_dir.join(name);
    std::fs::create_dir_all(&prof_dir).map_err(|e| e.to_string())?;
    let path = prof_dir.join("oauth.json");
    let json = serde_json::to_string_pretty(account).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}
