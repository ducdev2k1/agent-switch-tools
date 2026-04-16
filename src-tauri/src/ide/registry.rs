use serde::{Deserialize, Serialize};

/// Supported IDE types (excludes Claude Code which has its own flow)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdeType {
    Cursor,
    Antigravity,
    Windsurf,
}

/// How to extract email from IDE auth data
pub enum EmailKeySource {
    /// Email stored directly in a separate key (e.g. Cursor: "cursorAuth/cachedEmail")
    DirectKey(&'static str),
    /// Email embedded in a JSON value (e.g. Antigravity: key="antigravityAuthStatus", field="email")
    JsonField(&'static str, &'static str),
    /// Email embedded in a base64-encoded protobuf field within a JSON key
    /// (e.g. Windsurf: key="windsurfAuthStatus", json_field="userStatusProtoBinaryBase64")
    ProtoBase64Email(&'static str, &'static str),
}

/// Configuration metadata for each IDE
pub struct IdeConfig {
    pub display_name: &'static str,
    /// App directory name used in OS-specific paths
    pub app_dir_name: &'static str,
    /// Auth keys to backup/restore from state.vscdb ItemTable
    pub auth_keys: &'static [&'static str],
    /// How to extract the user's email from auth data
    pub email_key: EmailKeySource,
    /// Process names to detect if IDE is running (for pgrep)
    pub process_names: &'static [&'static str],
    /// CLI command name to launch/reload the IDE
    pub cli_command: &'static str,
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
    /// Return all supported IDE types
    pub fn all() -> &'static [IdeType] {
        &[IdeType::Cursor, IdeType::Antigravity, IdeType::Windsurf]
    }

    /// Get configuration for this IDE type
    pub fn config(&self) -> IdeConfig {
        match self {
            IdeType::Cursor => IdeConfig {
                display_name: "Cursor",
                app_dir_name: "Cursor",
                auth_keys: &[
                    "cursorAuth/accessToken",
                    "cursorAuth/refreshToken",
                    "cursorAuth/cachedEmail",
                    "cursorAuth/cachedSignUpType",
                    "cursorAuth/stripeMembershipType",
                ],
                email_key: EmailKeySource::DirectKey("cursorAuth/cachedEmail"),
                process_names: &["cursor", "Cursor"],
                cli_command: "cursor",
            },
            IdeType::Antigravity => IdeConfig {
                display_name: "Antigravity",
                app_dir_name: "Antigravity",
                auth_keys: &[
                    "antigravityAuthStatus",
                    "antigravityUnifiedStateSync.oauthToken",
                ],
                email_key: EmailKeySource::JsonField("antigravityAuthStatus", "email"),
                process_names: &["antigravity", "Antigravity"],
                cli_command: "antigravity",
            },
            IdeType::Windsurf => IdeConfig {
                display_name: "Windsurf",
                app_dir_name: "Windsurf",
                auth_keys: &[
                    "windsurfAuthStatus",
                    "codeium.windsurf-windsurf_auth",
                ],
                email_key: EmailKeySource::ProtoBase64Email(
                    "windsurfAuthStatus",
                    "userStatusProtoBinaryBase64",
                ),
                process_names: &["windsurf", "Windsurf"],
                cli_command: "windsurf",
            },
        }
    }

    /// Lowercase string ID for storage paths and serialization
    pub fn id(&self) -> &'static str {
        match self {
            IdeType::Cursor => "cursor",
            IdeType::Antigravity => "antigravity",
            IdeType::Windsurf => "windsurf",
        }
    }

    /// Parse IDE type from string (case-insensitive)
    pub fn from_str(s: &str) -> Result<IdeType, String> {
        match s.to_lowercase().as_str() {
            "cursor" => Ok(IdeType::Cursor),
            "antigravity" => Ok(IdeType::Antigravity),
            "windsurf" => Ok(IdeType::Windsurf),
            _ => Err(format!("Unknown IDE type: {}", s)),
        }
    }
}
