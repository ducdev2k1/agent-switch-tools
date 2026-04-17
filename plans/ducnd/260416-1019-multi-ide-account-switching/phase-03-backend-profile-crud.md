# Phase 3: Backend — Profile CRUD per IDE

**Priority:** High
**Status:** Pending
**Depends on:** Phase 1, Phase 2

---

## Overview

Tao Tauri commands cho list/save/switch/rename/delete profiles cua Cursor va Antigravity. Pattern tuong tu `config_commands.rs` hien tai nhung dung SQLite thay vi plaintext JSON.

## Key Insights

- Moi IDE co rieng profiles dir: `~/.claude/.claude-tools/{ide}/profiles/{email}/auth-keys.json`
- Moi IDE co rieng `meta.json`: `~/.claude/.claude-tools/{ide}/meta.json`
- Flow switch: backup current auth keys -> restore target auth keys -> update meta
- Can check IDE running truoc khi switch (IDE phai dong de ghi state.vscdb)

## Architecture

Profile storage per IDE:
```
~/.claude/.claude-tools/cursor/
  meta.json                    { activeProfileName, lastSwitchedAt, usageHistory }
  profiles/
    user1@gmail.com/
      auth-keys.json           { "cursorAuth/accessToken": "...", ... }
    user2@company.com/
      auth-keys.json

~/.claude/.claude-tools/antigravity/
  meta.json
  profiles/
    user3@gmail.com/
      auth-keys.json           { "antigravityAuthStatus": "...", ... }
```

## Related Code Files

**Create:**
- `src-tauri/src/commands/ide_profile_commands.rs` — Tauri commands for IDE profiles

**Read for context:**
- `src-tauri/src/commands/config_commands.rs` — existing Claude Code profile commands (pattern to follow)
- `src-tauri/src/commands/metadata_commands.rs` — existing meta read/write
- `src-tauri/src/commands/ide_registry.rs` (Phase 1)
- `src-tauri/src/commands/ide_sqlite_credentials.rs` (Phase 2)

## Implementation Steps

1. Create `ide_profile_commands.rs` with Tauri commands:

   ```rust
   #[tauri::command]
   pub async fn list_ide_profiles(app: AppHandle, ide_type: String) -> Result<Vec<IdeProfile>>
   // - Read current auth from state.vscdb (active profile)
   // - Scan profiles dir for saved profiles
   // - Return merged list with isActive flag
   
   #[tauri::command]  
   pub async fn save_current_ide_profile(app: AppHandle, ide_type: String) -> Result<String>
   // - Read auth keys from state.vscdb
   // - Extract email as profile name
   // - Save auth-keys.json to profiles dir
   // - Update meta.json
   
   #[tauri::command]
   pub async fn switch_ide_profile(app: AppHandle, ide_type: String, target_name: String) -> Result<IdeSwitchResult>
   // - Check IDE running -> warn
   // - Backup current auth keys to their profile dir
   // - Read target auth-keys.json
   // - Write target keys into state.vscdb
   // - Update meta.json
   
   #[tauri::command]
   pub async fn rename_ide_profile(app: AppHandle, ide_type: String, old_name: String, new_name: String) -> Result<()>
   
   #[tauri::command]
   pub async fn delete_ide_profile(app: AppHandle, ide_type: String, name: String) -> Result<()>
   
   #[tauri::command]
   pub async fn is_ide_running(ide_type: String) -> Result<bool>
   // - pgrep -f for process names
   ```

2. Define response types:
   ```rust
   struct IdeProfile {
       name: String,
       is_active: bool,
       email: Option<String>,
       membership_type: Option<String>,  // Cursor: stripeMembershipType
       display_name: Option<String>,     // Antigravity: name from authStatus
       ide_type: String,
   }
   
   struct IdeSwitchResult {
       success: bool,
       ide_was_running: bool,
       message: String,
   }
   ```

3. Reuse existing `ManagerMeta` struct for per-IDE meta.json (same format)

4. Register all commands in `mod.rs` and `lib.rs`

## Todo

- [ ] Create `ide_profile_commands.rs`
- [ ] Implement `list_ide_profiles`
- [ ] Implement `save_current_ide_profile`
- [ ] Implement `switch_ide_profile`
- [ ] Implement `rename_ide_profile` + `delete_ide_profile`
- [ ] Implement `is_ide_running`
- [ ] Register commands in mod.rs + lib.rs
- [ ] Test with real Cursor/Antigravity data on dev machine

## Success Criteria

- Can list profiles for Cursor and Antigravity
- Can save current IDE account as profile
- Can switch between profiles (IDE must be closed)
- Graceful errors when IDE running or DB locked
- Meta tracking works per IDE
