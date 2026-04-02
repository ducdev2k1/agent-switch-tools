mod commands;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Credential profile management
            commands::config_commands::list_credential_profiles,
            commands::config_commands::save_current_as_profile,
            commands::config_commands::switch_credential_profile,
            commands::config_commands::rename_credential_profile,
            commands::config_commands::delete_credential_profile,
            commands::config_commands::get_claude_cli_state,
            commands::config_commands::is_claude_running,
            // Metadata
            commands::metadata_commands::get_manager_meta,
            commands::metadata_commands::set_active_profile_name,
            // OAuth account
            commands::oauth_commands::get_oauth_account,
            // Usage stats
            commands::quota_commands::get_usage_stats,
            commands::quota_commands::get_usage_limits,
            commands::quota_commands::get_profile_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
