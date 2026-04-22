use crate::modules::providers::claude_cli::config::{self, ManagerMeta};
use crate::modules::shared::paths::claude_data_dir;

#[tauri::command]
pub async fn get_manager_meta(app: tauri::AppHandle) -> Result<ManagerMeta, String> {
    let dir = claude_data_dir(&app)?;
    Ok(config::read_meta(&dir))
}

#[tauri::command]
pub async fn set_active_profile_name(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let dir = claude_data_dir(&app)?;
    let mut meta = config::read_meta(&dir);
    meta.active_profile_name = Some(name);
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    config::write_meta(&dir, &meta)
}
