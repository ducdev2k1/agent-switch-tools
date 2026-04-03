use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

use crate::commands::metadata_commands::read_meta;

/// Setup system tray with profile quick-switch menu
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    // Build initial tray menu
    let menu = build_tray_menu(&handle)?;

    // Load platform-appropriate icon from bundle config (icns on macOS, ico on Windows, png on Linux)
    let icon = app.default_window_icon().cloned();
    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Claude Tools")
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
                _ if id.starts_with("switch:") => {
                    let profile_name = id.strip_prefix("switch:").unwrap_or("");
                    if !profile_name.is_empty() {
                        // Emit event to frontend to trigger switch via IPC
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
    let tools_dir = home.join(".claude").join(".claude-tools");
    let profiles_dir = tools_dir.join("profiles");
    let meta = read_meta(&tools_dir);

    let active_name = meta
        .active_profile_name
        .unwrap_or_else(|| "Active".to_string());

    let mut builder = MenuBuilder::new(handle);

    // Header
    let header = MenuItemBuilder::with_id("header", "Claude Tools")
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

    // Scan saved profiles: ~/.claude/.claude-tools/profiles/{name}/credentials.json
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

    builder = builder.separator();

    // Open dashboard
    let open = MenuItemBuilder::with_id("open", "Open Dashboard").build(handle)?;
    builder = builder.item(&open);

    // Quit
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(handle)?;
    builder = builder.item(&quit);

    Ok(builder.build()?)
}
