# Phase 1: Core Switch Logic Fixes

**Priority:** High | **Status:** Pending

## Overview

Fix fundamental issues in credential switching: track active profile name, atomic file operations, proper UI dialogs, token validation, cleanup dead code.

## Key Insights

- Active profile is always named "Active" — no way to know which saved profile it came from
- `switch_credential_profile` requires `current_name` but frontend hacks with `window.prompt`
- If rename step 2 fails, credentials end up in inconsistent state (no active file)
- `profile_commands.rs` is dead code (not registered in lib.rs)

## Related Files

**Modify:**
- `src-tauri/src/commands/config_commands.rs` — switch logic, metadata tracking
- `src-tauri/src/lib.rs` — command registration
- `src/hooks/use-profiles.ts` — simplified switch API
- `src/pages/dashboard.tsx` — remove window.prompt hack
- `src/lib/types.ts` — add active profile name field

**Delete:**
- `src-tauri/src/commands/profile_commands.rs` — dead code

**Create:**
- `src-tauri/src/commands/metadata_commands.rs` — profile metadata store (active name tracking)

## Implementation Steps

### 1. Add Profile Metadata Store

Store active profile name in `~/.claude/.claude-manager-meta.json`:
```json
{
  "activeProfileName": "Work",
  "lastSwitchedAt": "2026-04-01T20:00:00Z"
}
```

In `metadata_commands.rs`:
- `read_manager_meta()` — read/create meta file
- `write_manager_meta()` — update meta file
- `get_active_profile_name()` — Tauri command
- `set_active_profile_name(name)` — Tauri command

### 2. Fix Switch Operation (Atomic)

Current flow (BROKEN on step 2 failure):
```
1. rename .credentials.json → .credentials-[current].json
2. rename .credentials-[target].json → .credentials.json  ← if this fails, no active!
```

Fixed flow:
```
1. copy .credentials-[target].json → .credentials.json.tmp
2. rename .credentials.json → .credentials-[current].json
3. rename .credentials.json.tmp → .credentials.json
4. update meta: activeProfileName = target
```

If step 3 fails, `.credentials.json.tmp` exists → recovery possible.

### 3. Simplify Switch API

Current: `switch_credential_profile(current_name, target_name)` — frontend must figure out current name.

New: `switch_credential_profile(target_name)` — backend reads current name from metadata.

```rust
#[tauri::command]
pub async fn switch_credential_profile(
    app: tauri::AppHandle,
    target_name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let meta = read_manager_meta(&dir);
    let current_name = meta.active_profile_name
        .unwrap_or_else(|| "Unnamed".to_string());
    // ... atomic swap logic
}
```

### 4. Token Expiry Validation

Before switching, check if target credential is expired:
```rust
fn is_credential_expired(info: &CredentialInfo) -> bool {
    info.expires_at
        .map(|exp| exp < chrono::Utc::now().timestamp_millis())
        .unwrap_or(false)
}
```

Add warning in switch response if expired (but still allow — token refresh may fix it).

### 5. Auto-Save on First Switch

When switching for the first time and active has no saved backup:
- Backend auto-saves with metadata name or "Default" fallback
- No more `window.prompt` — use proper SaveProfileDialog in frontend

### 6. Update Frontend Types

```typescript
// types.ts - add to CredentialProfile
export interface CredentialProfile {
  name: string
  isActive: boolean
  isExpired: boolean      // NEW
  info: CredentialInfo
}
```

### 7. Remove Dead Code

Delete `src-tauri/src/commands/profile_commands.rs` and any imports in `lib.rs`.

## Todo

- [ ] Create `metadata_commands.rs` with meta file read/write
- [ ] Register new commands in `lib.rs`
- [ ] Refactor `switch_credential_profile` — single param, atomic swap
- [ ] Add `is_expired` field to `CredentialProfile`
- [ ] Update `list_credential_profiles` to include expiry check + active name from meta
- [ ] Update `save_current_as_profile` to also update meta
- [ ] Remove `window.prompt` from `dashboard.tsx`, use dialog
- [ ] Delete `profile_commands.rs`
- [ ] Update `use-profiles.ts` — simplified `switchTo(targetName)` API

## Success Criteria

- Switch requires only target profile name (no current_name)
- Active profile shows its saved name (not just "Active")
- Switch is atomic — no inconsistent state on partial failure
- Expired credentials show warning badge
- No dead code remains
