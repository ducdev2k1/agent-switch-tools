use crate::commands::path_helpers;
use crate::modules::usage::models::UsageReport;
use crate::modules::usage::report::build_report;

/// Build the Claude Code cost/usage report for the given date range.
/// `range_days`: 7 / 30 / 90, or 0 for all-time.
#[tauri::command]
pub async fn get_usage(app: tauri::AppHandle, range_days: u32) -> Result<UsageReport, String> {
    let claude_dir = path_helpers::claude_dir(&app)?;
    let cache_dir = path_helpers::claude_tools_dir(&app)?;
    Ok(build_report(&claude_dir, &cache_dir, range_days).await)
}
