use serde::Serialize;
use std::path::PathBuf;

use super::path_helpers;
use super::webhook_commands::WebhookResponse;

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

/// Response payload sent to webhook
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUsagePayload {
    pub event: String,
    pub timestamp: String,
    pub app_version: String,
    pub device_info: Option<serde_json::Value>,
    pub member_email: Option<String>,
    pub period: String,
    pub summary: AggregateSummary,
    pub sessions: Vec<SessionUsageSummary>,
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
fn parse_session_logs(
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

/// Send session token usage data to a custom webhook endpoint.
/// `period` can be: "1h", "5h", "24h", "7d", or "session" (current/latest session only)
#[tauri::command]
pub async fn send_session_usage_webhook(
    app: tauri::AppHandle,
    url: String,
    secret: Option<String>,
    period: Option<String>,
    member_email: Option<String>,
    detail_level: Option<String>,
) -> Result<WebhookResponse, String> {
    // Validate URL
    if !url.starts_with("https://")
        && !url.starts_with("http://localhost")
        && !url.starts_with("http://127.0.0.1")
    {
        return Ok(WebhookResponse {
            success: false,
            status_code: None,
            message: "Invalid URL: HTTPS required (localhost allowed for testing)".to_string(),
        });
    }

    let claude_dir = path_helpers::claude_dir(&app)?;
    let period_str = period.unwrap_or_else(|| "24h".to_string());
    let detail = detail_level.unwrap_or_else(|| "detailed".to_string());

    // Calculate since timestamp from period
    let since = match period_str.as_str() {
        "1h" => chrono::Utc::now() - chrono::Duration::hours(1),
        "5h" => chrono::Utc::now() - chrono::Duration::hours(5),
        "24h" => chrono::Utc::now() - chrono::Duration::hours(24),
        "7d" => chrono::Utc::now() - chrono::Duration::days(7),
        _ => chrono::Utc::now() - chrono::Duration::hours(24),
    };

    let sessions = parse_session_logs(&claude_dir, since);

    // Build aggregate summary
    let summary = AggregateSummary {
        total_input_tokens: sessions.iter().map(|s| s.total_input_tokens).sum(),
        total_output_tokens: sessions.iter().map(|s| s.total_output_tokens).sum(),
        total_cache_read: sessions.iter().map(|s| s.total_cache_read).sum(),
        total_cache_write: sessions.iter().map(|s| s.total_cache_write).sum(),
        session_count: sessions.len() as u64,
    };

    // Build device info
    let device_info = super::path_helpers::claude_tools_dir(&app)
        .and_then(|dir| super::device_commands::ensure_device_info(&dir).map_err(|e| e.to_string()))
        .ok()
        .map(|dev| {
            serde_json::json!({
                "device_id": dev.device_id,
                "device_name": dev.device_name,
                "hostname": dev.hostname,
            })
        });

    // Build payload — include sessions only if detail level is "detailed" or "per_session"
    let include_sessions = detail != "summary";

    let payload = SessionUsagePayload {
        event: "session_usage_report".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        device_info,
        member_email,
        period: period_str,
        summary,
        sessions: if include_sessions { sessions } else { Vec::new() },
    };

    // Send HTTP POST
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut req = client.post(&url).header("Content-Type", "application/json");

    if let Some(ref s) = secret {
        if !s.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", s));
        }
    }

    let res = match req.json(&payload).send().await {
        Ok(r) => r,
        Err(e) => {
            let msg = if e.is_timeout() {
                "Connection timed out (10s)".to_string()
            } else if e.is_connect() {
                "Connection refused".to_string()
            } else {
                format!("Request failed: {}", e)
            };
            return Ok(WebhookResponse {
                success: false,
                status_code: None,
                message: msg,
            });
        }
    };

    let status = res.status();
    let status_code = status.as_u16();

    if status.is_success() {
        Ok(WebhookResponse {
            success: true,
            status_code: Some(status_code),
            message: format!("OK ({}) — {} sessions sent", status_code, payload.sessions.len()),
        })
    } else {
        let body = res.text().await.unwrap_or_default();
        let msg = if body.len() > 200 {
            format!("HTTP {} — {}...", status_code, &body[..200])
        } else if body.is_empty() {
            format!("HTTP {}", status_code)
        } else {
            format!("HTTP {} — {}", status_code, body)
        };
        Ok(WebhookResponse {
            success: false,
            status_code: Some(status_code),
            message: msg,
        })
    }
}
