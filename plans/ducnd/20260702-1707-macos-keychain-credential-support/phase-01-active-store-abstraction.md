# Phase 01 — ActiveStore abstraction + Keychain/file backends + blob-core refactor

## Context links
- `src-tauri/src/modules/shared/paths.rs:12-14` (`claude_dir`)
- `src-tauri/src/modules/shared/mod.rs` (module registration)
- `src-tauri/src/modules/providers/claude_cli/oauth.rs:43,70,98,122,171` (refresh + file IO)
- `src-tauri/src/modules/providers/claude_cli/config.rs:85-113,137-143` (`read_credential_info`, `set_file_600`)
- `src-tauri/src/commands/quota_commands.rs:151-158` (`read_token_from_creds`)

## Overview
- Priority: P1. Status: pending.
- Foundation phase: create the abstraction and split file IO from pure logic. **No call site
  changes here** — Linux/Windows keep the exact file path unchanged. Behavior-neutral refactor.

## Key insights
- Every active touchpoint currently derives one path: `claude_dir(app).join(".credentials.json")`.
- 4 functions consume that path with file IO baked in: `oauth::ensure_fresh_token`,
  `oauth::force_refresh_token`, `config::read_credential_info`, `quota_commands::read_token_from_creds`.
- Keychain has no path/`exists()`/`fs::copy` — so those helpers must accept a **JSON blob**, not a path.
- The stored Keychain value is the identical JSON blob (`{"claudeAiOauth":{...}}`) as the file.

## Requirements
Functional:
- `ActiveStore` with: `read_active() -> Option<String>`, `write_active(&str) -> Result<(),String>`,
  `active_exists() -> bool`, `delete_active() -> Result<(),String>`.
- macOS backend: Keychain-first read, file fallback; write per hybrid policy below.
- Linux/Windows backend: file at `~/.claude/.credentials.json` (same as today).
- Blob-based cores so refresh/parse/token-read work independent of storage.

Non-functional: no new deps (preferred); secret never in process argv; file writes keep 0o600.

## Architecture

### New module `src-tauri/src/modules/shared/active_store.rs`
```
pub struct ActiveStore {
    config_dir: PathBuf,   // ~/.claude
    creds_file: PathBuf,   // <config_dir>/.credentials.json
}

impl ActiveStore {
    pub fn new(config_dir: PathBuf) -> Self;                 // used by resolve() + unit tests
    pub fn resolve(app: &tauri::AppHandle) -> Result<Self, String>; // config_dir = claude_dir(app)

    pub fn read_active(&self) -> Option<String>;            // JSON blob
    pub fn write_active(&self, blob: &str) -> Result<(), String>;
    pub fn active_exists(&self) -> bool;                     // = read_active().is_some()
    pub fn delete_active(&self) -> Result<(), String>;

    fn use_keychain() -> bool { cfg!(target_os = "macos") } // runtime gate, compile-safe
    fn hashed_service(&self) -> String;  // "Claude Code-credentials-"+sha256(config_dir)[:8]
}
```
Free helpers (compile on every OS; only run on macOS via the `cfg!` gate):
`keychain_suffix(dir)->String`, `read_keychain_blob(service)->Option<String>`,
`keychain_service_exists(service)->bool`, `write_keychain_blob(service,account,blob)->bool`,
`delete_keychain(service)`, `read_file_blob(path)`, `write_file_blob(path,blob)`, `current_user()`.

**File backend (Linux/Windows + macOS fallback):** `read_file_blob` = `fs::read_to_string` (None if
empty); `write_file_blob` = atomic temp+rename, `OpenOptions.mode(0o600)` under `#[cfg(unix)]`,
preserve perms on overwrite; delete = `fs::remove_file` (ignore NotFound).

**Keychain backend (`security` CLI, macOS only via `cfg!`):**
- `read_active`: if `use_keychain()` → try `read_keychain_blob(hashed_service())`, then
  `read_keychain_blob("Claude Code-credentials")`; on both empty fall through to
  `read_file_blob(creds_file)`. Off-macOS → straight to file.
  - `read_keychain_blob`: `security find-generic-password -s <service> -w`; success + non-empty
    stdout (trimmed) → `Some`, else `None`.
- `write_active`: if `use_keychain()` → pick service = hashed if it exists, else global if it
  exists, else hashed (modern default). `write_keychain_blob` = 
  `security add-generic-password -U -A -s <service> -a <account> -w <blob>`, **read back by service
  and confirm** blob matches, **retry ≤3× / 200ms**. Then **mirror to file only if creds_file
  already exists**. If keychain write fails but file exists, write file as fallback. Off-macOS →
  `write_file_blob`.
- `active_exists`: `read_active().is_some()`.
- `delete_active`: if `use_keychain()` → `security delete-generic-password -s <service>` for both
  hashed and global (ignore not-found). Always remove creds_file if present.

Account resolution `current_user()`: `std::env::var("USER")` → fallback parse of the keychain
attribute dump (`"acct"...="<name>"`, split on first `=`) → fallback `"$USER"` literal is avoided;
default to the login name. The `-a` account must match what Claude Code created; on upsert `-U`
the (service) match is what actually lands the write, confirmed by the read-back.

### Blob-core refactor (behavior-neutral)
`oauth.rs`:
- Rename `load_creds(path)` internals: add `fn parse_creds(content: &str) -> Result<Creds, String>`;
  keep `load_creds(path)` = `read_to_string` + `parse_creds` (profiles still use it).
- Add `fn serialize_root(root: &Value) -> Result<String, String>` (pretty).
- Extract refresh mutation from `perform_refresh` (oauth.rs:70) into
  `async fn refresh_into(root: &mut Value, refresh_token: &str) -> Result<String, String>`
  (does HTTP + writes 3 fields, returns access token, **no file IO**).
- `perform_refresh(path,...)` becomes: `refresh_into` + `write_atomic` (unchanged file path).

`config.rs`:
- Add `pub fn parse_credential_info(content: &str) -> CredentialInfo` (body of current fn minus the
  read); `read_credential_info(path)` = read file + `parse_credential_info`.

`quota_commands.rs`:
- Add `pub fn parse_token(blob: &str) -> Option<String>`; `read_token_from_creds(path)` wraps it.

## Related code files
Create:
- `src-tauri/src/modules/shared/active_store.rs`

Modify:
- `src-tauri/src/modules/shared/mod.rs` (add `pub mod active_store;`)
- `src-tauri/src/modules/providers/claude_cli/oauth.rs` (split parse/refresh; add nothing new to call sites)
- `src-tauri/src/modules/providers/claude_cli/config.rs` (add `parse_credential_info`)
- `src-tauri/src/commands/quota_commands.rs` (add `parse_token`)

Delete: none.

## Implementation steps
1. Add `active_store.rs` with the `ActiveStore` struct + File backend + all free helpers; register
   in `mod.rs`. Keychain helpers use `Command::new("security")` (compile on every OS).
2. Gate keychain use with runtime `cfg!(target_os = "macos")` inside `read_active`/`write_active`/
   `delete_active`. Only `#[cfg(unix)]` for the 0o600 `PermissionsExt` block.
3. Refactor `oauth.rs`: `parse_creds`, `serialize_root`, `refresh_into`; keep path fns green.
4. Add `config::parse_credential_info` + `quota_commands::parse_token`, wrap existing fns.
5. `cargo build` on Linux — must compile the FULL keychain branch and pass with zero call-site
   changes (File path unchanged, `cfg!` makes Linux take the file arm at runtime).

## Todo
- [ ] `ActiveStore` struct + File backend + free helpers
- [ ] Keychain read/write/delete gated by `cfg!(target_os="macos")` (compiles on Linux)
- [ ] Register module in `mod.rs`
- [ ] oauth.rs blob-core split
- [ ] config.rs `parse_credential_info`
- [ ] quota_commands.rs `parse_token`
- [ ] `cargo build` + `cargo test` clean on Linux

## Success criteria
- Linux build compiles the whole keychain branch (proves no syntax/type error off-Mac).
- Existing tests still pass (no behavior change on the file path).
- File backend `read/write/exists/delete` covered by unit tests (phase-04).

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Wrong keychain service name → writes a slot the CLI never reads (fake switch) | Med | High | Use the sha256-suffixed service (verified from ai-switcher); read-back confirm after write |
| `-A` grants any local process read access to the token | Low | Med | Documented trade-off; needed for unsigned build to avoid per-write prompts; personal-machine scope |
| Blob on argv briefly visible via `ps` | Low | Low | Short-lived; same approach a production macOS switcher ships; not logged anywhere |
| Blob refactor changes JSON formatting (pretty vs compact) affecting CLI | Low | Med | Keep `to_string_pretty` as today; snapshot-compare in tests |
| Keychain locked (DarkWake) → read empty | Med | Med | File fallback + mirror-to-file on write keeps a readable copy |

## Security considerations
- Blob is an OAuth secret: never logged; passed to `security` as `-w` arg (short-lived, matches the
  production switcher this technique is copied from).
- File backend retains 0o600 on write (temp file opened `mode(0o600)` before any bytes land).
- Keychain `-A`: item readable by any process of this user without a prompt — acceptable on a
  personal machine and required for an unsigned/ad-hoc-signed local build; same token Claude Code
  itself already stores.

## Next steps
Unblocks phase-02 (commands + reconcile) and phase-03 (refresh/quota/tray/priming).
</content>
