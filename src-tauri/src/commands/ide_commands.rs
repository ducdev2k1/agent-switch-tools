use crate::modules::providers::{IdeInfo, IdeType};
use crate::modules::core::ide_manager::{self, IdeProfile, IdeSwitchResult};
use crate::modules::core::path_helpers;

#[tauri::command]
pub async fn list_installed_ides(app: tauri::AppHandle) -> Result<Vec<IdeInfo>, String> {
    path_helpers::list_installed_ides(app).await
}

#[tauri::command]
pub async fn list_ide_profiles(
    app: tauri::AppHandle,
    ide_type: String,
) -> Result<Vec<IdeProfile>, String> {
    ide_manager::list_profiles(&app, &ide_type).await
}

#[tauri::command]
pub async fn save_current_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
) -> Result<String, String> {
    let email = ide_manager::save_current_profile(&app, &ide_type).await?;
    crate::tray::refresh_tray_menu(&app);
    Ok(email)
}

#[tauri::command]
pub async fn switch_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
    target_name: String,
) -> Result<IdeSwitchResult, String> {
    let result = ide_manager::switch_profile(&app, &ide_type, &target_name).await?;
    crate::tray::refresh_tray_menu(&app);
    Ok(result)
}

#[tauri::command]
pub async fn rename_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    ide_manager::rename_profile(&app, &ide_type, &old_name, &new_name).await?;
    crate::tray::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub async fn delete_ide_profile(
    app: tauri::AppHandle,
    ide_type: String,
    name: String,
) -> Result<(), String> {
    ide_manager::delete_profile(&app, &ide_type, &name).await?;
    crate::tray::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub async fn is_ide_running(ide_type: String) -> Result<bool, String> {
    let ide = IdeType::from_str(&ide_type)?;
    Ok(ide_manager::check_ide_running(&ide))
}

#[tauri::command]
pub async fn restart_ide(ide_type: String) -> Result<String, String> {
    ide_manager::restart_ide(&ide_type).await
}
