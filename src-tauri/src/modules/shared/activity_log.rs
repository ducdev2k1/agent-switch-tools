//! Append-only activity log shared by scheduled priming and the auto-switch rule.
//!
//! Both features record one pipe-delimited line per event and render it as a table.
//! The mechanics live here so the line cap is enforced in exactly one place: an
//! uncapped log grows without bound and every layer above it (IPC payload, parsing,
//! DOM rows) pays for every line ever written.
//!
//! Field parsing stays with each feature — the two logs carry different columns, and
//! a forced common schema would only obscure that.

use std::io::Write;
use std::path::Path;

/// Rewrite the file once it exceeds this many lines.
const MAX_LINES: usize = 5_000;
/// Lines kept (the newest ones) when a rewrite happens.
const KEEP_LINES: usize = 2_000;
/// Counting lines means reading the file, so only do it once the file is big enough
/// that it could plausibly be over MAX_LINES.
const SIZE_CHECK_BYTES: u64 = 256 * 1024;

/// Timestamp prefix every line starts with.
fn stamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Append one event: `{timestamp} | {field} | {field} | ...`.
/// Errors are swallowed — losing an activity-log line must never fail the operation
/// that was being logged.
pub fn append(path: &Path, fields: &[&str]) {
    let line = format!("{} | {}\n", stamp(), fields.join(" | "));
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
    enforce_cap(path);
}

/// Truncate to the newest KEEP_LINES when the file grew past MAX_LINES.
/// Safe to call on a missing or already-small file.
pub fn enforce_cap(path: &Path) {
    match std::fs::metadata(path) {
        Ok(m) if m.len() > SIZE_CHECK_BYTES => {}
        _ => return,
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= MAX_LINES {
        return;
    }
    let kept = lines[lines.len() - KEEP_LINES..].join("\n");
    let _ = std::fs::write(path, format!("{kept}\n"));
}

/// All lines oldest-first (empty when the file is missing). For whole-log aggregates.
pub fn lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .map(|c| c.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

/// One page of lines, newest first, plus the total line count so the UI can paginate.
pub fn page(path: &Path, offset: usize, limit: usize) -> (Vec<String>, usize) {
    let all = lines(path);
    let total = all.len();
    let rows = all
        .into_iter()
        .rev()
        .skip(offset)
        .take(limit)
        .collect();
    (rows, total)
}

/// Split a log line into its trimmed fields. The first field is always the timestamp.
/// Trailing fields may themselves contain " | ", so callers cap the split with `max_fields`.
pub fn split_line(line: &str, max_fields: usize) -> Vec<String> {
    line.splitn(max_fields, '|')
        .map(|p| p.trim().to_string())
        .collect()
}
