use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_sessions: usize,
    pub recent_sessions_7d: usize,
    pub current_model: Option<String>,
    pub has_restrictions: bool,
}

#[tauri::command]
pub async fn get_usage_stats(app: tauri::AppHandle) -> Result<UsageStats, String> {
    let home = app.path().home_dir().map_err(|e| e.to_string())?;
    let claude_dir = home.join(".claude");

    let history_path = claude_dir.join("history.jsonl");
    let settings_path = claude_dir.join("settings.json");
    let policy_path = claude_dir.join("policy-limits.json");

    let (total_sessions, recent_sessions_7d) = parse_history(&history_path);
    let current_model = read_model_from_settings(&settings_path);
    let has_restrictions = check_policy_restrictions(&policy_path);

    Ok(UsageStats {
        total_sessions,
        recent_sessions_7d,
        current_model,
        has_restrictions,
    })
}

/// Parse history.jsonl để đếm session tổng và 7 ngày gần đây
fn parse_history(path: &PathBuf) -> (usize, usize) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
    let mut total = 0;
    let mut recent = 0;

    for line in content.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            total += 1;
            // history.jsonl lưu timestamp dạng milliseconds
            if let Some(ts) = v.get("timestamp").and_then(|t| t.as_i64()) {
                let ts_secs = ts / 1000;
                if let Some(dt) = chrono::DateTime::from_timestamp(ts_secs, 0) {
                    if dt > cutoff {
                        recent += 1;
                    }
                }
            }
        }
    }

    (total, recent)
}

/// Đọc model từ settings.json
fn read_model_from_settings(path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("model")?.as_str().map(String::from)
}

/// Kiểm tra policy restrictions
fn check_policy_restrictions(path: &PathBuf) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(restrictions) = v.get("restrictions").and_then(|r| r.as_object()) {
            return restrictions.values().any(|v| {
                v.get("allowed").and_then(|a| a.as_bool()) == Some(false)
            });
        }
    }
    false
}

// Placeholder cho Anthropic Quota API (chưa có public endpoint)
#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub requests_limit: Option<u64>,
    pub requests_used: Option<u64>,
    pub tokens_limit: Option<u64>,
    pub tokens_used: Option<u64>,
    pub reset_at: Option<String>,
}

#[tauri::command]
pub async fn fetch_anthropic_quota(_api_key: String) -> Result<Option<QuotaInfo>, String> {
    // Hiện tại (2026-04) chưa có public quota endpoint
    // Trả về None để UI hiển thị "N/A"
    Ok(None)
}
