use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::path_helpers;

/// Token usage attributed to one model within a session.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelTokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

/// Aggregated usage for a single Claude Code session
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUsageSummary {
    pub session_id: String,
    pub project: String,
    pub branch: String,
    /// Dominant model of the session (most tokens, `<synthetic>` excluded).
    pub model: String,
    pub started_at: String,
    pub ended_at: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read: u64,
    pub total_cache_write: u64,
    pub message_count: u64,
    /// Per-model breakdown — a session can span several models (main model,
    /// subagents, mid-session /model switches).
    pub by_model: HashMap<String, ModelTokenUsage>,
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
    let mut first_model = String::new();
    let mut by_model: HashMap<String, ModelTokenUsage> = HashMap::new();
    // Streaming rewrites the same assistant message on several lines, each
    // carrying the identical usage object — count each message.id once.
    let mut seen_message_ids: HashSet<String> = HashSet::new();
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

        let message = parsed.get("message");
        let line_model = message
            .and_then(|msg| msg.get("model"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if first_model.is_empty() && !line_model.is_empty() {
            first_model = line_model.to_string();
        }

        // Extract usage from assistant messages, once per unique message id
        if let Some(usage) = message.and_then(|m| m.get("usage")) {
            if let Some(id) = message
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_str())
            {
                if !seen_message_ids.insert(id.to_string()) {
                    continue;
                }
            }

            let input = usage.get("input_tokens").and_then(|v| v.as_u64());
            let output = usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_read = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_write = usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if input.is_some() {
                has_usage = true;
            }
            let input = input.unwrap_or(0);

            total_input += input;
            total_output += output;
            total_cache_read += cache_read;
            total_cache_write += cache_write;
            message_count += 1;

            if !line_model.is_empty() {
                let entry = by_model.entry(line_model.to_string()).or_default();
                entry.input += input;
                entry.output += output;
                entry.cache_read += cache_read;
                entry.cache_write += cache_write;
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
        model: dominant_model(&by_model, &first_model),
        started_at: first_ts,
        ended_at: last_ts,
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_cache_read,
        total_cache_write,
        message_count,
        by_model,
    })
}

/// Model with the most tokens, ignoring `<synthetic>` placeholder entries
/// unless nothing else carries usage. Falls back to the first model seen.
fn dominant_model(by_model: &HashMap<String, ModelTokenUsage>, first_model: &str) -> String {
    let weight = |m: &ModelTokenUsage| m.input + m.output + m.cache_read + m.cache_write;
    by_model
        .iter()
        .filter(|(name, _)| *name != "<synthetic>")
        .max_by_key(|(_, m)| weight(m))
        .or_else(|| by_model.iter().max_by_key(|(_, m)| weight(m)))
        .map(|(name, _)| name.clone())
        .unwrap_or_else(|| first_model.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn line(id: &str, model: &str, input: u64, output: u64) -> String {
        format!(
            r#"{{"timestamp":"2026-07-02T01:00:00Z","message":{{"id":"{}","model":"{}","usage":{{"input_tokens":{},"output_tokens":{},"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#,
            id, model, input, output
        )
    }

    fn parse_fixture(content: &str) -> SessionUsageSummary {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("session.jsonl");
        std::fs::write(&path, content).unwrap();
        let since = chrono::DateTime::from_timestamp(0, 0).unwrap();
        parse_single_session(&path, "proj", since).expect("summary")
    }

    /// Streaming rewrites the same message on several lines with identical
    /// usage — it must be counted exactly once.
    #[test]
    fn duplicate_message_ids_are_counted_once() {
        let content = [
            line("msg_1", "claude-opus-4-8", 100, 50),
            line("msg_1", "claude-opus-4-8", 100, 50),
            line("msg_1", "claude-opus-4-8", 100, 50),
            line("msg_2", "claude-opus-4-8", 10, 5),
        ]
        .join("\n");

        let s = parse_fixture(&content);
        assert_eq!(s.total_input_tokens, 110);
        assert_eq!(s.total_output_tokens, 55);
        assert_eq!(s.message_count, 2);
    }

    /// Tokens must be attributed to the model that produced them, and the
    /// session label must be the dominant model — not the first line's model.
    #[test]
    fn tokens_split_per_model_and_dominant_wins() {
        let content = [
            line("msg_1", "claude-haiku-4-5-20251001", 10, 5),
            line("msg_2", "claude-opus-4-8", 1000, 500),
            line("msg_3", "claude-opus-4-8", 1000, 500),
        ]
        .join("\n");

        let s = parse_fixture(&content);
        assert_eq!(s.model, "claude-opus-4-8");
        assert_eq!(s.by_model.len(), 2);
        assert_eq!(s.by_model["claude-haiku-4-5-20251001"].input, 10);
        assert_eq!(s.by_model["claude-opus-4-8"].input, 2000);
    }

    /// `<synthetic>` placeholder messages must never name the session.
    #[test]
    fn synthetic_never_labels_the_session() {
        let content = [
            line("msg_1", "<synthetic>", 5000, 100),
            line("msg_2", "claude-sonnet-4-6", 10, 5),
        ]
        .join("\n");

        let s = parse_fixture(&content);
        assert_eq!(s.model, "claude-sonnet-4-6");
        assert!(s.by_model.contains_key("<synthetic>"));
    }
}
