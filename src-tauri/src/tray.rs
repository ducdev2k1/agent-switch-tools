use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

use crate::modules::providers::claude_cli::config;
use crate::modules::core::path_helpers::{ide_db_path, ide_is_installed, ide_profiles_dir, ide_tools_dir};
use crate::modules::providers::IdeType;
use crate::modules::core::sqlite_auth::read_ide_auth_keys;

/// Setup system tray with profile quick-switch menu
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    // Build initial tray menu
    let menu = build_tray_menu(&handle)?;

    // Load platform-appropriate icon from bundle config (icns on macOS, ico on Windows, png on Linux)
    let icon = app.default_window_icon().cloned();
    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Agent Switch Tools")
        .menu(&menu)
        .show_menu_on_left_click(true);
    if let Some(icon) = icon {
        builder = builder.icon(icon).icon_as_template(true); // icon_as_template for macOS dark/light bar
    }
    builder
        .on_menu_event(move |app_handle: &tauri::AppHandle, event: tauri::menu::MenuEvent| {
            let id = event.id().as_ref();
            match id {
                "open" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app_handle.exit(0);
                }
                _ if id.starts_with("ide-switch:") => {
                    // Format: ide-switch:{ideType}:{profileName}
                    let rest = id.strip_prefix("ide-switch:").unwrap_or("");
                    if let Some((ide_type, profile_name)) = rest.split_once(':') {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let payload = format!("{}:{}", ide_type, profile_name);
                        let _ = app_handle.emit("tray-switch-ide-profile", payload);
                    }
                }
                _ if id.starts_with("switch:") => {
                    let profile_name = id.strip_prefix("switch:").unwrap_or("");
                    if !profile_name.is_empty() {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = app_handle.emit("tray-switch-profile", profile_name);
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

/// Rebuild tray menu after profile changes (save/switch/delete)
pub fn refresh_tray_menu(handle: &tauri::AppHandle) {
    if let Ok(menu) = build_tray_menu(handle) {
        if let Some(tray) = handle.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

/// Build tray menu dynamically from saved profiles
fn build_tray_menu(
    handle: &tauri::AppHandle,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let home = handle.path().home_dir()?;
    let claude_dir = home.join(".agent-switch-tools").join("claude");
    let profiles_dir = claude_dir.join("profiles");
    let meta = config::read_meta(&claude_dir);

    let active_name = meta
        .active_profile_name
        .unwrap_or_else(|| "Active".to_string());

    let mut builder = MenuBuilder::new(handle);

    // Header
    let header = MenuItemBuilder::with_id("header", "Agent Switch Tools")
        .enabled(false)
        .build(handle)?;
    builder = builder.item(&header);
    builder = builder.separator();

    // Active profile indicator
    let active_label = format!("✓ {} (active)", active_name);
    let active_item = MenuItemBuilder::with_id("active", &active_label)
        .enabled(false)
        .build(handle)?;
    builder = builder.item(&active_item);

    // Scan saved profiles: ~/.agent-switch-tools/profiles/{name}/credentials.json
    if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| !name.is_empty() && name != &active_name)
            .collect();
        names.sort();

        for name in names {
            let id = format!("switch:{}", name);
            let item = MenuItemBuilder::with_id(id, &format!("  {}", name)).build(handle)?;
            builder = builder.item(&item);
        }
    }

    // IDE sections (Cursor, Antigravity)
    for ide_type in IdeType::all() {
        if !ide_is_installed(handle, ide_type) {
            continue;
        }
        let provider = ide_type.provider();
        let ide_id = ide_type.id();

        builder = builder.separator();

        // IDE header
        let ide_header =
            MenuItemBuilder::with_id(format!("ide-header:{}", ide_id), provider.display_name())
                .enabled(false)
                .build(handle)?;
        builder = builder.item(&ide_header);

        // Read active account from IDE's state.vscdb, fallback to meta.json
        {
            let ide_active = ide_db_path(handle, ide_type)
                .ok()
                .and_then(|db_path| read_ide_auth_keys(&db_path, provider.auth_keys()).ok())
                .and_then(|auth_data| provider.extract_email(&auth_data))
                .or_else(|| {
                    ide_tools_dir(handle, ide_type)
                        .ok()
                        .and_then(|dir| config::read_meta(&dir).active_profile_name)
                })
                .unwrap_or_else(|| "Not logged in".to_string());

            let ide_active_label = format!("✓ {} (active)", ide_active);
            let ide_active_item = MenuItemBuilder::with_id(
                format!("ide-active:{}", ide_id),
                &ide_active_label,
            )
            .enabled(false)
            .build(handle)?;
            builder = builder.item(&ide_active_item);

            // Scan saved IDE profiles
            if let Ok(ide_profs) = ide_profiles_dir(handle, ide_type) {
                if let Ok(entries) = std::fs::read_dir(&ide_profs) {
                    let mut ide_names: Vec<String> = entries
                        .flatten()
                        .filter(|e| e.path().is_dir())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .filter(|n| !n.is_empty() && n != &ide_active)
                        .collect();
                    ide_names.sort();

                    for ide_name in ide_names {
                        let item_id = format!("ide-switch:{}:{}", ide_id, ide_name);
                        let item =
                            MenuItemBuilder::with_id(item_id, &format!("  {}", ide_name))
                                .build(handle)?;
                        builder = builder.item(&item);
                    }
                }
            }
        }
    }

    builder = builder.separator();

    // Open dashboard
    let open = MenuItemBuilder::with_id("open", "Open Dashboard").build(handle)?;
    builder = builder.item(&open);

    // Quit
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(handle)?;
    builder = builder.item(&quit);

    Ok(builder.build()?)
}
