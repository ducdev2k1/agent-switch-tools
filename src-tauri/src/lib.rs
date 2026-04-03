mod commands;
mod quota_refresh_worker;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Non-fatal: tray may fail on Linux without libayatana-appindicator3
            if let Err(e) = tray::setup_tray(app) {
                eprintln!("Warning: Could not initialize system tray: {e}");
            }
            quota_refresh_worker::spawn_quota_worker(app.handle().clone());
            Ok(())
        })
        // Close to tray: hide window instead of quitting when user clicks X
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
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
