use std::collections::HashMap;
use super::IdeProvider;
use crate::modules::providers::utils::extract_plan_from_proto_json;

pub struct WindsurfProvider;

impl IdeProvider for WindsurfProvider {
    fn display_name(&self) -> &'static str { "Windsurf" }
    fn app_dir_name(&self) -> &'static str { "Windsurf" }
    fn auth_keys(&self) -> &'static [&'static str] {
        &[
            "windsurfAuthStatus",
            "codeium.windsurf-windsurf_auth",
        ]
    }
    fn token_key(&self) -> Option<&'static str> { Some("codeium.windsurf-windsurf_auth") }
    fn process_names(&self) -> &'static [&'static str] { &["windsurf", "Windsurf"] }
    fn cli_command(&self) -> &'static str { "windsurf" }

    fn extract_email(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        let auth_status = auth_data.get("windsurfAuthStatus")?;
        let v: serde_json::Value = serde_json::from_str(auth_status).ok()?;
        v.get("userStatusProtoBinaryBase64")?; // We just check it's parseable?
        // Wait, earlier logic was extracting plan from proto. Email was NOT from proto?
        // Registry said: ProtoBase64Email("windsurfAuthStatus", "userStatusProtoBinaryBase64")
        // But the parse code in profile_commands never extracted email from proto, it just left it as None, or used the name.
        // Actually earlier code:
        // windsurf doesn't have a clear email field in settings often unless we parse the proto correctly for email.
        None
    }

    fn extract_display_name(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        auth_data.get("codeium.windsurf-windsurf_auth")
            .cloned()
            .filter(|s| !s.is_empty() && s != "[]")
    }

    fn extract_membership(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        let json_str = auth_data.get("windsurfAuthStatus")?;
        let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
        extract_plan_from_proto_json(&v)
    }

    fn normalize_token(&self, raw_token: &str) -> String {
        if raw_token.starts_with('{') || raw_token.starts_with('[') {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw_token) {
                v.get("api_key")
                    .or_else(|| v.get("token"))
                    .or_else(|| v.get("accessToken"))
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or(raw_token.to_string())
            } else {
                raw_token.to_string()
            }
        } else {
            raw_token.to_string()
        }
    }
}
