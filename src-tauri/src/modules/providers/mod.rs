use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod antigravity;
pub mod claude_cli;
pub mod cursor;
pub mod utils;
pub mod windsurf;

/// Supported IDE types for profile management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdeType {
    Cursor,
    Antigravity,
    AntigravityIde,
    AntigravityCli,
    Windsurf,
}

/// Info returned to frontend about an installed IDE
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdeInfo {
    pub ide_type: IdeType,
    pub display_name: String,
    pub is_installed: bool,
}

impl IdeType {
    pub fn all() -> &'static [IdeType] {
        &[
            IdeType::Cursor,
            IdeType::Antigravity,
            IdeType::AntigravityIde,
            IdeType::AntigravityCli,
            IdeType::Windsurf,
        ]
    }

    /// IDEs hidden from every UI surface (tray menu + dashboard). They keep
    /// working internally — switching/quota still function — they are just not
    /// shown. Remove an arm here to re-enable an IDE in the UI.
    pub fn is_hidden(&self) -> bool {
        matches!(
            self,
            IdeType::Cursor
                | IdeType::Windsurf
                | IdeType::Antigravity
                | IdeType::AntigravityIde
                | IdeType::AntigravityCli
        )
    }

    pub fn id(&self) -> &'static str {
        match self {
            IdeType::Cursor => "cursor",
            IdeType::Antigravity => "antigravity",
            IdeType::AntigravityIde => "antigravity-ide",
            IdeType::AntigravityCli => "antigravity-cli",
            IdeType::Windsurf => "windsurf",
        }
    }

    pub fn from_str(s: &str) -> Result<IdeType, String> {
        match s.to_lowercase().as_str() {
            "cursor" => Ok(IdeType::Cursor),
            "antigravity" => Ok(IdeType::Antigravity),
            "antigravity-ide" => Ok(IdeType::AntigravityIde),
            "antigravity-cli" => Ok(IdeType::AntigravityCli),
            "windsurf" => Ok(IdeType::Windsurf),
            _ => Err(format!("Unknown IDE type: {}", s)),
        }
    }

    pub fn provider(&self) -> Box<dyn IdeProvider> {
        match self {
            IdeType::Cursor => Box::new(cursor::CursorProvider),
            IdeType::Antigravity => Box::new(antigravity::AntigravityProvider),
            IdeType::AntigravityIde => Box::new(antigravity::AntigravityIdeProvider),
            IdeType::AntigravityCli => Box::new(antigravity::AntigravityCliProvider),
            IdeType::Windsurf => Box::new(windsurf::WindsurfProvider),
        }
    }
}

pub trait IdeProvider: Send + Sync {
    // Configuration
    fn display_name(&self) -> &'static str;
    fn app_dir_name(&self) -> &'static str;
    fn auth_keys(&self) -> &'static [&'static str];
    fn token_key(&self) -> Option<&'static str>;
    fn process_names(&self) -> &'static [&'static str];
    fn cli_command(&self) -> &'static str;

    // Data Extraction
    fn extract_email(&self, auth_data: &HashMap<String, String>) -> Option<String>;
    fn extract_display_name(&self, auth_data: &HashMap<String, String>) -> Option<String>;
    fn extract_membership(&self, auth_data: &HashMap<String, String>) -> Option<String>;
    
    // Auth Data Normalization (e.g. parsing token raw from JSON string)
    fn normalize_token(&self, raw_token: &str) -> String {
        raw_token.to_string() // Default no-op
    }
}
