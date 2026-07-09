use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::StoreExt;

mod commands;
pub mod modules;
mod priming;
mod quota_refresh_worker;
mod tray;

/// Restore the main window after it was hidden (close-to-tray) or reopened via
/// the tray menu / single-instance relaunch.
pub(crate) fn present_main_window(window: &tauri::WebviewWindow) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

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
                present_main_window(&window);
            }
        }))
        .setup(|app| {
            // Migrate data from the pre-rebrand layout before anything reads it,
            // then clean up the old renamed app bundle (macOS; no-op elsewhere).
            if let Ok(home) = modules::shared::paths::home_dir(app.handle()) {
                modules::core::legacy_migration::migrate_legacy_data(&home);
                modules::core::legacy_uninstall::remove_legacy_app(&home);
            }
            if let Err(e) = tray::setup_tray(app) {
                eprintln!("Warning: Could not initialize system tray: {e}");
            }
            if let Ok(dir) = commands::path_helpers::claude_tools_dir(app.handle()) {
                if let Err(e) = commands::device_commands::ensure_device_info(&dir) {
                    eprintln!("Warning: Could not initialize device identity: {e}");
                }
            }
            // Window starts hidden (see tauri.conf.json) so we can decide whether to
            // show it before it ever paints, avoiding a flash when "start minimized" is on.
            if let Some(window) = app.get_webview_window("main") {
                let start_minimized = app
                    .store("settings.json")
                    .ok()
                    .and_then(|store| store.get("start_minimized"))
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                if !start_minimized {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            quota_refresh_worker::spawn_quota_worker(app.handle().clone());
            priming::scheduler::spawn_prime_scheduler(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            // Wayland/GTK bug: after a window is hidden and shown again, its native
            // title bar buttons (close/minimize/maximize) stop responding to clicks
            // until the window is resized. Toggling `resizable` on refocus forces
            // GTK to re-register the button handlers without a visible resize.
            // https://github.com/tauri-apps/tauri/issues/11856
            #[cfg(target_os = "linux")]
            tauri::WindowEvent::Focused(true) => {
                let _ = window.set_resizable(false);
                let _ = window.set_resizable(true);
            }
            _ => {}
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
            // Metadata
            commands::metadata_commands::get_manager_meta,
            commands::metadata_commands::set_active_profile_name,
            // Changelog
            commands::changelog_commands::fetch_changelog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
