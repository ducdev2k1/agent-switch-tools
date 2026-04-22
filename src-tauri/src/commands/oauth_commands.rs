use crate::modules::providers::claude_cli::auth::{self, OAuthAccount};
use crate::modules::shared::paths::home_dir;

#[tauri::command]
pub async fn get_claude_oauth_account(app: tauri::AppHandle) -> Result<OAuthAccount, String> {
    let home = home_dir(&app)?;
    auth::read_oauth_from_claude_json(&home)
        .ok_or_else(|| "No oauth account found in .claude.json".to_string())
}

#[tauri::command]
pub async fn save_oauth_account(
    app: tauri::AppHandle,
    name: String,
    account: OAuthAccount,
) -> Result<(), String> {
    let profs_dir = crate::modules::shared::paths::profiles_dir(&app)?;
    auth::write_saved_oauth(&profs_dir, &name, &account)
}

#[tauri::command]
pub async fn update_active_oauth(
    app: tauri::AppHandle,
    account: OAuthAccount,
) -> Result<(), String> {
    let home = home_dir(&app)?;
    auth::update_claude_json_oauth(&home, &account)
}
