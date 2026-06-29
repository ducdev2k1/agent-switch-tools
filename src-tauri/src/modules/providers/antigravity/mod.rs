pub mod oauth;
pub mod quota;

use std::collections::HashMap;
use super::IdeProvider;
use crate::modules::providers::utils::extract_plan_from_proto_json;

/// Synthetic auth key under which the Antigravity CLI's JSON token file content is stored.
pub const CLI_TOKEN_KEY: &str = "antigravityCliToken";
/// Cached email key persisted into saved profiles for variants that don't store email locally.
pub const CACHED_EMAIL_KEY: &str = "email";
/// Cached display-name key (same rationale as CACHED_EMAIL_KEY).
pub const CACHED_NAME_KEY: &str = "name";

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

/// Antigravity IDE — the newer, separately-branded build (`~/.config/Antigravity IDE`).
///
/// Unlike the original Antigravity it does NOT store `antigravityAuthStatus`. Auth lives
/// entirely in `antigravityUnifiedStateSync.oauthToken` (OAuth proto) and `userStatus`.
/// Email is not stored locally — it is resolved via Google userinfo and cached into the
/// saved profile under `email`/`name` (see ide_manager email resolution).
pub struct AntigravityIdeProvider;

impl IdeProvider for AntigravityIdeProvider {
    fn display_name(&self) -> &'static str { "Antigravity IDE" }
    fn app_dir_name(&self) -> &'static str { "Antigravity IDE" }
    fn auth_keys(&self) -> &'static [&'static str] {
        &[
            "antigravityUnifiedStateSync.oauthToken",
            "antigravityUnifiedStateSync.userStatus",
            CACHED_EMAIL_KEY,
            CACHED_NAME_KEY,
        ]
    }
    // Token is inside the oauthToken proto, not a flat JSON apiKey → handled by OAuth flow.
    fn token_key(&self) -> Option<&'static str> { None }
    fn process_names(&self) -> &'static [&'static str] { &["antigravity-ide", "Antigravity IDE"] }
    fn cli_command(&self) -> &'static str { "antigravity-ide" }

    fn extract_email(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        auth_data.get(CACHED_EMAIL_KEY).filter(|s| !s.is_empty()).cloned()
    }

    fn extract_display_name(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        auth_data.get(CACHED_NAME_KEY).filter(|s| !s.is_empty()).cloned()
    }

    fn extract_membership(&self, _auth_data: &HashMap<String, String>) -> Option<String> {
        // Plan/tier is not stored locally in this build; surfaced via quota API instead.
        None
    }
}

/// Antigravity CLI — Gemini-based CLI storing a plain JSON token file in `~/.gemini/antigravity-cli`.
///
/// Credentials are NOT a state.vscdb; the whole token file is exposed under CLI_TOKEN_KEY.
pub struct AntigravityCliProvider;

impl IdeProvider for AntigravityCliProvider {
    fn display_name(&self) -> &'static str { "Antigravity CLI" }
    // Unused for the CLI (it resolves a JsonFile credential source), kept for trait completeness.
    fn app_dir_name(&self) -> &'static str { "antigravity-cli" }
    fn auth_keys(&self) -> &'static [&'static str] {
        &[CLI_TOKEN_KEY, CACHED_EMAIL_KEY, CACHED_NAME_KEY]
    }
    fn token_key(&self) -> Option<&'static str> { None }
    fn process_names(&self) -> &'static [&'static str] { &["antigravity-cli"] }
    fn cli_command(&self) -> &'static str { "antigravity" }

    fn extract_email(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        auth_data.get(CACHED_EMAIL_KEY).filter(|s| !s.is_empty()).cloned()
    }

    fn extract_display_name(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        auth_data.get(CACHED_NAME_KEY).filter(|s| !s.is_empty()).cloned()
    }

    fn extract_membership(&self, _auth_data: &HashMap<String, String>) -> Option<String> {
        None
    }

    fn normalize_token(&self, raw_token: &str) -> String {
        // raw_token is the CLI token file JSON: { "token": { "access_token": "ya29.*", ... } }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw_token) {
            if let Some(tok) = v.get("token").and_then(|t| t.get("access_token")).and_then(|x| x.as_str()) {
                return tok.to_string();
            }
        }
        raw_token.to_string()
    }
}
