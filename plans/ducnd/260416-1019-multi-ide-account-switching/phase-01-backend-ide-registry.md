# Phase 1: Backend — IDE Registry + Path Resolver

**Priority:** High
**Status:** Pending
**Depends on:** None

---

## Overview

Tao IDE registry abstraction trong Rust backend. Moi IDE la mot enum variant voi metadata (ten, credential paths, auth keys, process name).

## Key Insights

- Cursor va Antigravity deu dung `state.vscdb` (SQLite) nhung o paths khac nhau va voi auth keys khac nhau
- Claude Code dung plaintext JSON — giu nguyen, khong thay doi
- Can cross-platform path resolution (Linux, macOS, Windows)

## Architecture

```rust
enum IdeType {
    Cursor,
    Antigravity,
}

struct IdeConfig {
    ide_type: IdeType,
    display_name: String,
    // Path to state.vscdb relative to OS app data
    db_relative_path: &'static str,
    // Auth keys to backup/restore from ItemTable
    auth_keys: Vec<&'static str>,
    // Key that contains email for profile naming
    email_key: EmailKeySource,
    // Process name for pgrep
    process_names: Vec<&'static str>,
}

enum EmailKeySource {
    DirectKey(&'static str),          // Cursor: "cursorAuth/cachedEmail"
    JsonField(&'static str, &'static str), // Antigravity: key="antigravityAuthStatus", field="email"
}
```

## Related Code Files

**Modify:**
- `src-tauri/src/commands/mod.rs` — register new module

**Create:**
- `src-tauri/src/commands/ide_registry.rs` — IdeType enum, IdeConfig, path resolver
- `src-tauri/src/commands/ide_path_helpers.rs` — cross-platform path resolution for IDE state.vscdb

**Read for context:**
- `src-tauri/src/commands/path_helpers.rs` — existing Claude Code path logic (DO NOT MODIFY)

## Implementation Steps

1. Create `ide_registry.rs`:
   - Define `IdeType` enum (Cursor, Antigravity) with serde Serialize/Deserialize
   - Define `IdeConfig` struct with all metadata per IDE
   - Implement `IdeType::config()` -> `IdeConfig` for each variant
   - Implement `IdeType::all()` -> Vec<IdeType>

2. Create `ide_path_helpers.rs`:
   - `ide_db_path(app, ide_type)` -> Result<PathBuf> — resolve absolute path to state.vscdb
     - Linux: `~/.config/{AppName}/User/globalStorage/state.vscdb`
     - macOS: `~/Library/Application Support/{AppName}/User/globalStorage/state.vscdb`
     - Windows: `%APPDATA%/{AppName}/User/globalStorage/state.vscdb`
   - `ide_profiles_dir(app, ide_type)` -> Result<PathBuf> — `~/.claude/.claude-tools/{ide}/profiles/`
   - `ide_tools_dir(app, ide_type)` -> Result<PathBuf> — `~/.claude/.claude-tools/{ide}/`
   - `ide_is_installed(app, ide_type)` -> bool — check if state.vscdb exists

3. Add Tauri command:
   - `list_installed_ides()` -> Vec<IdeInfo> — return installed IDEs with display name + type

4. Register in `mod.rs`

## IDE Config Data

```
Cursor:
  display_name: "Cursor"
  app_name: "Cursor"
  auth_keys: [
    "cursorAuth/accessToken",
    "cursorAuth/refreshToken", 
    "cursorAuth/cachedEmail",
    "cursorAuth/cachedSignUpType",
    "cursorAuth/stripeMembershipType"
  ]
  email_key: DirectKey("cursorAuth/cachedEmail")
  process_names: ["cursor", "Cursor"]

Antigravity:
  display_name: "Antigravity"
  app_name: "Antigravity"
  auth_keys: [
    "antigravityAuthStatus",
    "antigravityUnifiedStateSync.oauthToken"
  ]
  email_key: JsonField("antigravityAuthStatus", "email")
  process_names: ["antigravity", "Antigravity"]
```

## Todo

- [ ] Create `ide_registry.rs` with IdeType enum + IdeConfig
- [ ] Create `ide_path_helpers.rs` with cross-platform resolvers
- [ ] Add `list_installed_ides` Tauri command
- [ ] Register modules in `mod.rs`

## Success Criteria

- `IdeType::all()` returns Cursor + Antigravity
- `ide_db_path()` resolves correct paths per OS
- `ide_is_installed()` correctly detects installed IDEs
- `list_installed_ides` command works from frontend
