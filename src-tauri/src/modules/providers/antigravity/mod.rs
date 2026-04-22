pub mod oauth;
pub mod quota;

use std::collections::HashMap;
use super::IdeProvider;
use crate::modules::providers::utils::extract_plan_from_proto_json;

pub struct AntigravityProvider;

impl IdeProvider for AntigravityProvider {
    fn display_name(&self) -> &'static str { "Antigravity" }
    fn app_dir_name(&self) -> &'static str { "Antigravity" }
    fn auth_keys(&self) -> &'static [&'static str] {
        &[
            "antigravityAuthStatus",
            "antigravityUnifiedStateSync.oauthToken",
        ]
    }
    // JSON blob containing apiKey (ya29.* OAuth token) + email + name + userStatusProtoBinaryBase64
    fn token_key(&self) -> Option<&'static str> { Some("antigravityAuthStatus") }
    fn process_names(&self) -> &'static [&'static str] { &["antigravity", "Antigravity"] }
    fn cli_command(&self) -> &'static str { "antigravity" }

    fn extract_email(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        let json_str = auth_data.get("antigravityAuthStatus")?;
        let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
        v.get("email").and_then(|e| e.as_str()).map(String::from)
    }

    fn extract_display_name(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        let json_str = auth_data.get("antigravityAuthStatus")?;
        let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
        v.get("name").and_then(|n| n.as_str()).map(String::from)
    }

    fn extract_membership(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        let json_str = auth_data.get("antigravityAuthStatus")?;
        let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
        extract_plan_from_proto_json(&v)
    }

    fn normalize_token(&self, raw_token: &str) -> String {
        // raw_token is the full antigravityAuthStatus JSON; extract apiKey (ya29.* OAuth bearer)
        if raw_token.starts_with('{') {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw_token) {
                if let Some(key) = v.get("apiKey").and_then(|x| x.as_str()) {
                    return key.to_string();
                }
                if let Some(key) = v.get("api_key").and_then(|x| x.as_str()) {
                    return key.to_string();
                }
            }
        }
        raw_token.to_string()
    }
}
