use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Mirror of the `oauthAccount` object Claude Code writes into `~/.claude.json` on login.
/// `extra` round-trips any fields we don't model (seatTier, organizationType, …) so
/// re-writing the object on switch never drops data the CLI put there.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_extra_usage_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_role: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Read the account identity from `~/.claude.json`.
///
/// `oauthAccount` is the source of truth written by Claude Code itself on every
/// login — NOT `claudeAiOauth`, which only exists in `.credentials.json` (and was
/// mistakenly cached into `~/.claude.json` by older app versions).
pub fn read_oauth_from_claude_json(home_dir: &PathBuf) -> Option<OAuthAccount> {
    let path = home_dir.join(".claude.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let oauth = v.get("oauthAccount")?;
    serde_json::from_value(oauth.clone()).ok()
}

pub fn update_claude_json_oauth(home_dir: &PathBuf, account: &OAuthAccount) -> Result<(), String> {
    let path = home_dir.join(".claude.json");
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut v: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    v["oauthAccount"] = serde_json::to_value(account).map_err(|e| e.to_string())?;
    // Older app versions wrote a stale `claudeAiOauth` cache here; drop it so no
    // reader can ever confuse it with the real account identity again.
    if let Some(obj) = v.as_object_mut() {
        obj.remove("claudeAiOauth");
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_claude_json(dir: &std::path::Path, content: &str) -> PathBuf {
        let home = dir.to_path_buf();
        std::fs::write(home.join(".claude.json"), content).unwrap();
        home
    }

    /// Regression: identity must come from `oauthAccount` (written by Claude Code),
    /// never from the app's legacy `claudeAiOauth` cache.
    #[test]
    fn reads_email_from_oauth_account_not_legacy_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let home = write_claude_json(
            tmp.path(),
            r#"{
                "oauthAccount": { "emailAddress": "new@inet.vn", "seatTier": "team_tier_1" },
                "claudeAiOauth": { "emailAddress": "stale@inet.vn" }
            }"#,
        );

        let acc = read_oauth_from_claude_json(&home).expect("oauthAccount should parse");
        assert_eq!(acc.email_address.as_deref(), Some("new@inet.vn"));
        // Unknown fields survive into `extra` for lossless re-writes.
        assert_eq!(
            acc.extra.get("seatTier").and_then(|v| v.as_str()),
            Some("team_tier_1")
        );
    }

    /// Switching writes `oauthAccount` back and purges the legacy `claudeAiOauth` key.
    /// Unset optional fields must be omitted, not written as `null` — `~/.claude.json`
    /// is owned by Claude Code and we only replace what it would write itself.
    #[test]
    fn update_writes_oauth_account_and_removes_legacy_key() {
        let tmp = tempfile::tempdir().unwrap();
        let home = write_claude_json(
            tmp.path(),
            r#"{
                "someOtherSetting": true,
                "oauthAccount": { "emailAddress": "old@inet.vn" },
                "claudeAiOauth": { "emailAddress": "stale@inet.vn" }
            }"#,
        );

        let acc = read_oauth_from_claude_json(&home).unwrap();
        let mut target = acc.clone();
        target.email_address = Some("target@inet.vn".to_string());
        update_claude_json_oauth(&home, &target).unwrap();

        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(home.join(".claude.json")).unwrap())
                .unwrap();
        assert_eq!(
            v["oauthAccount"]["emailAddress"].as_str(),
            Some("target@inet.vn")
        );
        assert!(v.get("claudeAiOauth").is_none());
        assert_eq!(v["someOtherSetting"], serde_json::Value::Bool(true));
        assert!(
            v["oauthAccount"].get("displayName").is_none(),
            "unset Option fields must be omitted, not null"
        );
    }
}
