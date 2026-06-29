use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

mod commands;
pub mod modules;
mod priming;
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
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            if let Err(e) = tray::setup_tray(app) {
                eprintln!("Warning: Could not initialize system tray: {e}");
            }
            if let Ok(dir) = commands::path_helpers::claude_tools_dir(app.handle()) {
                if let Err(e) = commands::device_commands::ensure_device_info(&dir) {
                    eprintln!("Warning: Could not initialize device identity: {e}");
                }
            }
            quota_refresh_worker::spawn_quota_worker(app.handle().clone());
            priming::scheduler::spawn_prime_scheduler(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Claude Credential profile management
            commands::config_commands::list_credential_profiles,
            commands::config_commands::save_current_as_profile,
            commands::config_commands::switch_credential_profile,
            commands::config_commands::rename_credential_profile,
            commands::config_commands::delete_credential_profile,
            commands::config_commands::get_claude_cli_state,
            commands::config_commands::is_claude_running,
            // OAuth account
            commands::oauth_commands::get_claude_oauth_account,
            commands::oauth_commands::save_oauth_account,
            commands::oauth_commands::update_active_oauth,
            // Usage stats & Quota
            commands::quota_commands::get_usage_stats,
            commands::quota_commands::get_usage_limits,
            commands::quota_commands::get_profile_usage,
            commands::quota_commands::get_ide_usage,
            // IDE multi-account management
            commands::ide_commands::list_installed_ides,
            commands::ide_commands::list_ide_profiles,
            commands::ide_commands::save_current_ide_profile,
            commands::ide_commands::switch_ide_profile,
            commands::ide_commands::rename_ide_profile,
            commands::ide_commands::delete_ide_profile,
            commands::ide_commands::is_ide_running,
            commands::ide_commands::restart_ide,
            // Token refresh
            commands::token_refresh::refresh_active_token,
            commands::token_refresh::refresh_profile_token,
            // Webhook
            commands::webhook_commands::send_webhook,
            // Device identity
            commands::device_commands::get_device_info,
            commands::device_commands::rename_device,
            // Session usage
            commands::session_usage_commands::get_session_usage,
            // Cost/usage analytics
            commands::usage_report_commands::get_usage,
            // Scheduled priming
            commands::priming_commands::set_auto_prime,
            commands::priming_commands::set_auto_prime_all,
            commands::priming_commands::prime_now,
            commands::priming_commands::get_auto_prime_settings,
            commands::priming_commands::get_auto_prime_log,
            commands::priming_commands::get_auto_prime_stats,
            // System info
            commands::system_info_commands::get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
