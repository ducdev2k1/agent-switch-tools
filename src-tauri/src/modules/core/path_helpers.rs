use std::path::PathBuf;

use crate::modules::core::credential_source::CredentialSource;
use crate::modules::providers::{IdeInfo, IdeType};
use crate::modules::shared::paths::{antigravity_cli_token_path, claude_tools_dir, ide_app_dir};

/// Resolve absolute path to IDE's state.vscdb
pub fn ide_db_path(app: &tauri::AppHandle, ide_type: &IdeType) -> Result<PathBuf, String> {
    let provider = ide_type.provider();
    let app_dir = ide_app_dir(app, provider.app_dir_name())?;
    Ok(app_dir
        .join("User")
        .join("globalStorage")
        .join("state.vscdb"))
}

/// Resolve the live credential store for an agent.
/// Most agents use a `state.vscdb`; the Antigravity CLI uses a JSON token file.
pub fn ide_credential_source(
    app: &tauri::AppHandle,
    ide_type: &IdeType,
) -> Result<CredentialSource, String> {
    match ide_type {
        IdeType::AntigravityCli => {
            Ok(CredentialSource::JsonFile(antigravity_cli_token_path(app)?))
        }
        _ => Ok(CredentialSource::Vscdb(ide_db_path(app, ide_type)?)),
    }
}

/// Check if an IDE is installed by verifying its credential store exists
pub fn ide_is_installed(app: &tauri::AppHandle, ide_type: &IdeType) -> bool {
    ide_credential_source(app, ide_type)
        .map(|s| s.exists())
        .unwrap_or(false)
}

/// ~/.agent-switch-tools/{ide}/ — per-IDE root for app-managed data
pub fn ide_tools_dir(app: &tauri::AppHandle, ide_type: &IdeType) -> Result<PathBuf, String> {
    let dir = claude_tools_dir(app)?.join(ide_type.id());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// ~/.agent-switch-tools/{ide}/profiles/ — saved profiles for this IDE
pub fn ide_profiles_dir(app: &tauri::AppHandle, ide_type: &IdeType) -> Result<PathBuf, String> {
    let dir = ide_tools_dir(app, ide_type)?.join("profiles");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// List all installed IDEs with their info (non-command version; command lives in commands/ide_commands.rs)
pub async fn list_installed_ides(app: tauri::AppHandle) -> Result<Vec<IdeInfo>, String> {
    let mut ides = Vec::new();
    for ide_type in IdeType::all() {
        if ide_type.is_hidden() {
            continue;
        }
        let provider = ide_type.provider();
        ides.push(IdeInfo {
            ide_type: *ide_type,
            display_name: provider.display_name().to_string(),
            is_installed: ide_is_installed(&app, ide_type),
        });
    }
    Ok(ides)
}
