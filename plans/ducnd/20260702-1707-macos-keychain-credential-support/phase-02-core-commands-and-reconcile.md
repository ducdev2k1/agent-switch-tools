# Phase 02 — Integrate ActiveStore into 3 core commands + reconcile

## Context links
- `src-tauri/src/commands/config_commands.rs:11-71` (`list_credential_profiles`)
- `src-tauri/src/commands/config_commands.rs:73-114` (`save_current_as_profile`)
- `src-tauri/src/commands/config_commands.rs:116-195` (`switch_credential_profile`)
- `src-tauri/src/modules/providers/claude_cli/reconcile.rs:45-130` (`reconcile_active_profile`, `refresh_active_backup`)

## Overview
- Priority: P1. Status: pending. Depends on phase-01.
- Route the active credential through `ActiveStore`. Saved-profile files and `~/.claude.json`
  identity reads stay exactly as-is.

## Key insights
- All three commands + reconcile compute `let active_path = claude.join(".credentials.json");`
  then do `.exists()` / `fs::copy` / `read_credential_info(&active_path)`.
- Replace that local `active_path` with `let store = ActiveStore::resolve(&app)?;` (commands) or a
  passed-in `&ActiveStore` (reconcile — see signature change below).
- `reconcile_active_profile` currently takes `claude: &PathBuf` and derives active_path
  (reconcile.rs:51). It has no `AppHandle`. Change its signature to accept `store: &ActiveStore`
  instead of `claude`. Callers: config_commands.rs:24 and :139 (only 2 — both have `app`).

## Requirements
- `list`: active detection + info via store, not file.
- `save_current`: read active blob from store, write to profile file.
- `switch`: back up current active (read store → write profile file), then write target file blob
  into store (`write_active`).
- `reconcile`: existence + copy-from-active + live-read all via store.
- `~/.claude.json` identity via `auth::read_oauth_from_claude_json` — unchanged (still file).

## Architecture / data flow

### `list_credential_profiles` (config_commands.rs:39-48)
- `let store = ActiveStore::resolve(&app)?;`
- `if store.active_exists()` instead of `active_path.exists()`.
- `let info = config::parse_credential_info(&store.read_active().unwrap_or_default());`
  instead of `read_credential_info(&active_path)`.

### `save_current_as_profile` (config_commands.rs:79-96)
- `if !store.active_exists() { return Err("No active credentials found") }`.
- Replace `fs::copy(&active_path, &target_path)` with:
  `let blob = store.read_active().ok_or("cannot read active credentials")?;`
  `fs::write(&target_path, &blob)?; config::set_file_600(&target_path);`
- `read_oauth_from_claude_json(&home)` unchanged (identity from `~/.claude.json`).

### `switch_credential_profile` (config_commands.rs:125-172)
- Backup block (config_commands.rs:150-159): replace `fs::copy(&active_path, &backup_path)` with
  read store blob → `fs::write(backup_path, blob)` + `set_file_600`. Guard on `store.active_exists()`.
- Apply target (config_commands.rs:161): replace `fs::copy(&target_cred_path, &active_path)` with
  `let blob = fs::read_to_string(&target_cred_path)?; store.write_active(&blob)?;`
- `update_claude_json_oauth` unchanged (identity file).

### `reconcile_active_profile` (reconcile.rs:45-100)
- Signature: `pub fn reconcile_active_profile(home, store: &ActiveStore, profs_dir, claude_data)`
  (drop `claude: &PathBuf`).
- reconcile.rs:51-54: `if !store.active_exists() { return Ok((None,false)) }`.
- reconcile.rs:81 `fs::copy(&active_path, &backup_path)` → read store blob → `fs::write`.
- `refresh_active_backup` (reconcile.rs:105-130): param `active_path` → `store: &ActiveStore`;
  reconcile.rs:115 `fs::read(active_path)` → `store.read_active()` (as bytes/string).
- Update the 2 callers (config_commands.rs:24, :139) to build `store` and pass `&store`.
- Update reconcile unit tests (reconcile.rs:146-226) to construct `ActiveStore::file(claude.join(".credentials.json"))` — behavior identical on Linux.

## Related code files
Modify:
- `src-tauri/src/commands/config_commands.rs`
- `src-tauri/src/modules/providers/claude_cli/reconcile.rs`

Create/Delete: none.

## Implementation steps
1. In each command build `let store = ActiveStore::resolve(&app)?;` once at top.
2. Swap `active_path.exists()` → `store.active_exists()`, reads → `store.read_active()`,
   writes-to-active → `store.write_active()`; keep saved-profile file writes as `fs::write`.
3. Change `reconcile_active_profile` + `refresh_active_backup` signatures to `&ActiveStore`;
   fix the 2 call sites.
4. Update reconcile tests to pass `ActiveStore::file(...)`.
5. `cargo build && cargo test` on Linux (file backend == old behavior).

## Todo
- [ ] `list_credential_profiles` via store
- [ ] `save_current_as_profile` via store
- [ ] `switch_credential_profile` via store (backup + apply)
- [ ] `reconcile_active_profile` signature + body
- [ ] `refresh_active_backup` signature + body
- [ ] Fix 2 reconcile callers + reconcile tests
- [ ] Linux build + tests green

## Success criteria
- Linux: switch/save/list behave identically to pre-change (verified by existing + new tests).
- No remaining `claude.join(".credentials.json")` for active in these 4 functions.

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Missed a hidden active-path read in these files | Low | Med | grep `credentials.json` in both files post-edit; must be profile-only |
| `write_active` partial failure leaves inconsistent state (Keychain written, meta not) | Low | Med | Write store BEFORE meta write; on store error return early (meta untouched) |
| reconcile signature change breaks other callers | Low | Low | Only 2 callers (verified); compiler enforces |

## Security considerations
- Backup files under `profiles/` keep 0o600 via `set_file_600` (already present).
- Active blob only held in memory transiently; not logged.

## Next steps
Unblocks phase-03. `~/.claude.json` identity assumption must be confirmed on Mac before phase-03
refresh testing (see plan Open Questions).
</content>
