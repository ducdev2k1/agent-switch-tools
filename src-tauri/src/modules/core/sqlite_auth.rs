use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time::Duration;


/// Read auth key-value pairs from IDE's state.vscdb (ItemTable)
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
        let value: Option<String> = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = ?1",
                rusqlite::params![key],
                |row| {
                    row.get::<_, String>(0)
                        .or_else(|_| {
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



fn open_db_readonly_with_retry(db_path: &Path) -> Result<rusqlite::Connection, String> {
    let open = || {
        let conn = rusqlite::Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| format!("Failed to open database: {}", e))?;
        conn.busy_timeout(Duration::from_secs(2))
            .map_err(|e| format!("Failed to set busy timeout: {}", e))?;
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

fn open_db_with_retry(db_path: &Path) -> Result<rusqlite::Connection, String> {
    let open = || {
        let conn = rusqlite::Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .map_err(|e| format!("Failed to open database: {}", e))?;
        conn.busy_timeout(Duration::from_secs(2))
            .map_err(|e| format!("Failed to set busy timeout: {}", e))?;
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
