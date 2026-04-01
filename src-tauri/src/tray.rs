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

    TrayIconBuilder::new()
        .tooltip("Claude Account Manager")
        .menu(&menu)
        .show_menu_on_left_click(true)
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

/// Build tray menu dynamically from saved profiles
fn build_tray_menu(
    handle: &tauri::AppHandle,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let home = handle.path().home_dir()?;
    let claude_dir = home.join(".claude");
    let meta = read_meta(&claude_dir);

    let active_name = meta
        .active_profile_name
        .unwrap_or_else(|| "Active".to_string());

    let mut builder = MenuBuilder::new(handle);

    // Header
    let header = MenuItemBuilder::with_id("header", "Claude Account Manager")
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

    // Scan saved profiles
    if let Ok(entries) = std::fs::read_dir(&claude_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with(".credentials-") && filename.ends_with(".json") {
                let name = filename
                    .strip_prefix(".credentials-")
                    .and_then(|s| s.strip_suffix(".json"))
                    .unwrap_or("")
                    .to_string();

                if !name.is_empty() {
                    let id = format!("switch:{}", name);
                    let item =
                        MenuItemBuilder::with_id(id, &format!("  {}", name)).build(handle)?;
                    builder = builder.item(&item);
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
