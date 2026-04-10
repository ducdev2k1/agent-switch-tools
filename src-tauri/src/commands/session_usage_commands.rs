use serde::Serialize;
use std::path::PathBuf;

use super::path_helpers;

/// Aggregated usage for a single Claude Code session
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUsageSummary {
    pub session_id: String,
    pub project: String,
    pub branch: String,
    pub model: String,
    pub started_at: String,
    pub ended_at: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read: u64,
    pub total_cache_write: u64,
    pub message_count: u64,
}

/// High-level aggregate across all sessions
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregateSummary {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read: u64,
    pub total_cache_write: u64,
    pub session_count: u64,
}

/// Parse all JSONL session files from ~/.claude/projects/ for a given period
pub fn parse_session_logs(
    claude_dir: &PathBuf,
    since: chrono::DateTime<chrono::Utc>,
) -> Vec<SessionUsageSummary> {
    let projects_dir = claude_dir.join("projects");
    if !projects_dir.exists() {
        return Vec::new();
    }

    let mut sessions: Vec<SessionUsageSummary> = Vec::new();

    // Iterate all project directories
    let project_dirs = match std::fs::read_dir(&projects_dir) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    for project_entry in project_dirs.flatten() {
        let project_path = project_entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let project_name = project_entry
            .file_name()
            .to_string_lossy()
            .to_string();

        // Find all .jsonl files in this project dir
        let entries = match std::fs::read_dir(&project_path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            // Skip subagent logs (inside subdirectories)
            if path.parent() != Some(&project_path) {
                continue;
            }

            // Check file modification time — skip old files
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    let mod_time: chrono::DateTime<chrono::Utc> = modified.into();
                    if mod_time < since {
                        continue;
                    }
                }
            }

            if let Some(summary) = parse_single_session(&path, &project_name, since) {
                sessions.push(summary);
            }
        }
    }

    // Sort by started_at ascending
    sessions.sort_by(|a, b| a.started_at.cmp(&b.started_at));
    sessions
}

/// Build aggregate summary from sessions
pub fn build_aggregate(sessions: &[SessionUsageSummary]) -> AggregateSummary {
    AggregateSummary {
        total_input_tokens: sessions.iter().map(|s| s.total_input_tokens).sum(),
        total_output_tokens: sessions.iter().map(|s| s.total_output_tokens).sum(),
        total_cache_read: sessions.iter().map(|s| s.total_cache_read).sum(),
        total_cache_write: sessions.iter().map(|s| s.total_cache_write).sum(),
        session_count: sessions.len() as u64,
    }
}

/// Parse a single .jsonl session file and aggregate token usage
fn parse_single_session(
    path: &PathBuf,
    project_name: &str,
    since: chrono::DateTime<chrono::Utc>,
) -> Option<SessionUsageSummary> {
    let content = std::fs::read_to_string(path).ok()?;

    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut total_input: u64 = 0;
    let mut total_output: u64 = 0;
    let mut total_cache_read: u64 = 0;
    let mut total_cache_write: u64 = 0;
    let mut message_count: u64 = 0;
    let mut first_ts = String::new();
    let mut last_ts = String::new();
    let mut branch = String::new();
    let mut model = String::new();
    let mut has_usage = false;

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Extract timestamp for filtering
        if let Some(ts_str) = parsed.get("timestamp").and_then(|t| t.as_str()) {
            // Filter by since
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                let ts_utc: chrono::DateTime<chrono::Utc> = ts.into();
                if ts_utc < since {
                    continue;
                }
            }

            if first_ts.is_empty() {
                first_ts = ts_str.to_string();
            }
            last_ts = ts_str.to_string();
        }

        // Extract branch from user messages
        if branch.is_empty() {
            if let Some(b) = parsed.get("gitBranch").and_then(|v| v.as_str()) {
                branch = b.to_string();
            }
        }

        // Extract usage from assistant messages
        if let Some(usage) = parsed
            .get("message")
            .and_then(|m| m.get("usage"))
        {
            if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                total_input += input;
                has_usage = true;
            }
            if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                total_output += output;
            }
            if let Some(cr) = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
            {
                total_cache_read += cr;
            }
            if let Some(cw) = usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
            {
                total_cache_write += cw;
            }
            message_count += 1;
        }

        // Extract model
        if model.is_empty() {
            if let Some(m) = parsed
                .get("message")
                .and_then(|msg| msg.get("model"))
                .and_then(|v| v.as_str())
            {
                model = m.to_string();
            }
        }
    }

    if !has_usage {
        return None;
    }

    Some(SessionUsageSummary {
        session_id,
        project: project_name.to_string(),
        branch,
        model,
        started_at: first_ts,
        ended_at: last_ts,
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_cache_read,
        total_cache_write,
        message_count,
    })
}

/// Get session usage summaries for a given period (hours back from now)
#[tauri::command]
pub async fn get_session_usage(
    app: tauri::AppHandle,
    hours_back: Option<u64>,
) -> Result<Vec<SessionUsageSummary>, String> {
    let claude_dir = path_helpers::claude_dir(&app)?;
    let hours = hours_back.unwrap_or(24);
    let since = chrono::Utc::now() - chrono::Duration::hours(hours as i64);
    Ok(parse_session_logs(&claude_dir, since))
}
