use std::collections::HashMap;
use super::IdeProvider;

pub struct CursorProvider;

impl IdeProvider for CursorProvider {
    fn display_name(&self) -> &'static str { "Cursor" }
    fn app_dir_name(&self) -> &'static str { "Cursor" }
    fn auth_keys(&self) -> &'static [&'static str] {
        &[
            "cursorAuth/accessToken",
            "cursorAuth/refreshToken",
            "cursorAuth/cachedEmail",
            "cursorAuth/cachedSignUpType",
            "cursorAuth/stripeMembershipType",
        ]
    }
    fn token_key(&self) -> Option<&'static str> { Some("cursorAuth/accessToken") }
    fn process_names(&self) -> &'static [&'static str] { &["cursor", "Cursor"] }
    fn cli_command(&self) -> &'static str { "cursor" }

    fn extract_email(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        auth_data.get("cursorAuth/cachedEmail").cloned()
    }

    fn extract_display_name(&self, _auth_data: &HashMap<String, String>) -> Option<String> {
        None
    }

    fn extract_membership(&self, auth_data: &HashMap<String, String>) -> Option<String> {
        auth_data.get("cursorAuth/stripeMembershipType").cloned()
    }
}
