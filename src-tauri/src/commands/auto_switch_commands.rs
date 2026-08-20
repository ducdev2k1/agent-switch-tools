use tauri::AppHandle;

use crate::auto_switch::{store, AutoSwitchConfig, AutoSwitchLogEntry};

/// Current auto-switch rule configuration (defaults when never configured).
#[tauri::command]
pub fn get_auto_switch_config(app: AppHandle) -> AutoSwitchConfig {
    store::load(&app)
}

/// Persist the rule configuration; threshold and cooldown are clamped in the store.
#[tauri::command]
pub fn set_auto_switch_config(app: AppHandle, config: AutoSwitchConfig) -> Result<(), String> {
    store::save(&app, &config)
}

/// Automatic-switch activity, newest entry first.
#[tauri::command]
pub fn get_auto_switch_history(app: AppHandle) -> Vec<AutoSwitchLogEntry> {
    store::read_history(&app)
}
