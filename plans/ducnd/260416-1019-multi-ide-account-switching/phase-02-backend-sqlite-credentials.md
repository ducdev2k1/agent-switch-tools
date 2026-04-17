# Phase 2: Backend — SQLite Credential Read/Write

**Priority:** High
**Status:** Pending
**Depends on:** Phase 1

---

## Overview

Them `rusqlite` crate va tao module doc/ghi auth keys tu `state.vscdb` (SQLite) cua Cursor va Antigravity.

## Key Insights

- `state.vscdb` co 1 table: `ItemTable (key TEXT UNIQUE, value BLOB)`
- Antigravity co the co them table `cursorDiskKV` (Cursor co)
- Auth values la UTF-8 text (JSON hoac JWT), luu dang BLOB
- state.vscdb co the bi lock boi IDE dang chay — can handle gracefully
- File size lon (300MB+ cho Cursor) nhung chi can doc/ghi vai keys

## Requirements

**Functional:**
- Doc tat ca auth keys tu state.vscdb cua mot IDE
- Ghi (replace) auth keys vao state.vscdb cua mot IDE
- Extract email tu auth data (khac nhau cho moi IDE)

**Non-functional:**
- Read-only access default, write chi khi switch
- Handle locked DB gracefully (retry/warn)
- Khong doc toan bo DB, chi query specific keys

## Related Code Files

**Modify:**
- `src-tauri/Cargo.toml` — add `rusqlite` dependency

**Create:**
- `src-tauri/src/commands/ide_sqlite_credentials.rs` — read/write auth keys

**Read for context:**
- `src-tauri/src/commands/ide_registry.rs` (Phase 1)
- `src-tauri/src/commands/config_commands.rs` — existing credential read pattern

## Implementation Steps

1. Add `rusqlite` to `Cargo.toml`:
   ```toml
   [dependencies]
   rusqlite = { version = "0.31", features = ["bundled"] }
   ```

2. Create `ide_sqlite_credentials.rs`:

   ```rust
   /// Read all auth key-value pairs from IDE's state.vscdb
   pub fn read_ide_auth_keys(db_path: &Path, keys: &[&str]) -> Result<HashMap<String, String>>

   /// Write auth key-value pairs into IDE's state.vscdb (UPSERT)
   pub fn write_ide_auth_keys(db_path: &Path, entries: &HashMap<String, String>) -> Result<()>

   /// Extract email from IDE auth data
   pub fn extract_ide_email(ide_type: &IdeType, auth_data: &HashMap<String, String>) -> Option<String>
   ```

3. Handle edge cases:
   - DB file not found -> return clear error
   - DB locked (IDE running) -> retry 2x with 500ms delay, then return error with message "Close {IDE} first"
   - Key not found -> skip (account may not be logged in)
   - Invalid UTF-8 in BLOB -> skip key with warning

4. Register in `mod.rs`

## Cargo.toml Change

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
```

Note: `bundled` feature compiles SQLite from source, avoiding system dependency issues.

## Todo

- [ ] Add `rusqlite` to Cargo.toml
- [ ] Create `ide_sqlite_credentials.rs`
- [ ] Implement `read_ide_auth_keys`
- [ ] Implement `write_ide_auth_keys`
- [ ] Implement `extract_ide_email`
- [ ] Handle DB locked / not found edge cases
- [ ] Register module in `mod.rs`

## Success Criteria

- Can read Cursor auth keys from state.vscdb on this machine
- Can read Antigravity auth keys from state.vscdb on this machine
- Graceful error when DB locked or missing
- Email extraction works for both IDEs
