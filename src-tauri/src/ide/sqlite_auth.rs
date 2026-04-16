use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time::Duration;

use super::registry::{EmailKeySource, IdeType};

/// Read auth key-value pairs from IDE's state.vscdb (ItemTable)
/// Opens in READ-ONLY mode so it works even while IDE is running.
pub fn read_ide_auth_keys(
    db_path: &Path,
    keys: &[&str],
) -> Result<HashMap<String, String>, String> {
    if !db_path.exists() {
        return Err(format!("Database not found: {}", db_path.display()));
    }

    let conn = open_db_readonly_with_retry(db_path)?;
    let mut result = HashMap::new();

    for key in keys {
        // Values may be stored as TEXT or BLOB — try String first, fallback to Vec<u8>
        let value: Option<String> = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = ?1",
                rusqlite::params![key],
                |row| {
                    // Try reading as String (TEXT type)
                    row.get::<_, String>(0)
                        .or_else(|_| {
                            // Fallback: read as BLOB and convert
                            row.get::<_, Vec<u8>>(0)
                                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                        })
                },
            )
            .ok();

        if let Some(text) = value {
            if !text.is_empty() {
                result.insert(key.to_string(), text);
            }
        }
    }

    Ok(result)
}

/// Write auth key-value pairs into IDE's state.vscdb (UPSERT into ItemTable)
/// Uses INSERT OR REPLACE to update existing keys or insert new ones.
pub fn write_ide_auth_keys(
    db_path: &Path,
    entries: &HashMap<String, String>,
) -> Result<(), String> {
    if !db_path.exists() {
        return Err(format!("Database not found: {}", db_path.display()));
    }

    let conn = open_db_with_retry(db_path)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    for (key, value) in entries {
        tx.execute(
            "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value.as_str()],
        )
        .map_err(|e| format!("Failed to write key '{}': {}", key, e))?;
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit: {}", e))?;

    Ok(())
}

/// Extract email from IDE auth data based on IDE-specific logic
pub fn extract_ide_email(
    ide_type: &IdeType,
    auth_data: &HashMap<String, String>,
) -> Option<String> {
    let config = ide_type.config();
    match &config.email_key {
        EmailKeySource::DirectKey(key) => auth_data.get(*key).cloned(),
        EmailKeySource::JsonField(key, field) => {
            let json_str = auth_data.get(*key)?;
            let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
            v.get(*field)?.as_str().map(String::from)
        }
        EmailKeySource::ProtoBase64Email(key, json_field) => {
            // Decode base64 protobuf and search for email pattern in raw bytes
            let json_str = auth_data.get(*key)?;
            let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
            let b64 = v.get(*json_field)?.as_str()?;
            let decoded = data_encoding::BASE64.decode(b64.as_bytes()).ok()?;
            let text = String::from_utf8_lossy(&decoded);
            // Simple email regex search in decoded proto bytes
            extract_email_from_text(&text)
        }
    }
}

/// Extract first email address from text using simple pattern matching
fn extract_email_from_text(text: &str) -> Option<String> {
    // Find email-like patterns: chars@chars.chars
    let mut best: Option<String> = None;
    for word in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '@' && c != '.' && c != '+' && c != '-' && c != '_') {
        if word.contains('@') && word.contains('.') {
            let trimmed = word.trim_matches('.');
            if trimmed.len() > 5 {
                best = Some(trimmed.to_string());
                break;
            }
        }
    }
    best
}

/// Open SQLite database in READ-ONLY mode with retry logic.
/// Safe to call while IDE is running (WAL mode allows concurrent reads).
fn open_db_readonly_with_retry(db_path: &Path) -> Result<rusqlite::Connection, String> {
    let open = || {
        let conn = rusqlite::Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| format!("Failed to open database: {}", e))?;

        conn.busy_timeout(Duration::from_secs(2))
            .map_err(|e| format!("Failed to set busy timeout: {}", e))?;

        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ItemTable'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| format!("Database verification failed: {}", e))?;

        Ok::<_, String>(conn)
    };

    for attempt in 0..3 {
        match open() {
            Ok(conn) => return Ok(conn),
            Err(e) if attempt < 2 && e.contains("locked") => {
                thread::sleep(Duration::from_millis(500));
            }
            Err(e) => return Err(e),
        }
    }

    Err("Database is locked. Please close the IDE and try again.".to_string())
}

/// Open SQLite database in READ-WRITE mode with retry logic.
/// Used only when writing (switch profile).
fn open_db_with_retry(db_path: &Path) -> Result<rusqlite::Connection, String> {
    let open = || {
        let conn = rusqlite::Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .map_err(|e| format!("Failed to open database: {}", e))?;

        // Set busy timeout to 2 seconds for WAL mode compatibility
        conn.busy_timeout(Duration::from_secs(2))
            .map_err(|e| format!("Failed to set busy timeout: {}", e))?;

        // Verify we can actually query by checking table exists
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ItemTable'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| format!("Database verification failed: {}", e))?;

        Ok::<_, String>(conn)
    };

    // Try up to 3 times with 500ms delay between retries
    for attempt in 0..3 {
        match open() {
            Ok(conn) => return Ok(conn),
            Err(e) if attempt < 2 && e.contains("locked") => {
                thread::sleep(Duration::from_millis(500));
            }
            Err(e) => return Err(e),
        }
    }

    Err("Database is locked. Please close the IDE and try again.".to_string())
}
