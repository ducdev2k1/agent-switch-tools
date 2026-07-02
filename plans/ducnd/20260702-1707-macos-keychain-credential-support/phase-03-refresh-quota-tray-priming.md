# Phase 03 — Refresh + quota worker/commands + tray + priming via ActiveStore

## Context links
- `src-tauri/src/modules/providers/claude_cli/oauth.rs:98,122` (`ensure_fresh_token`, `force_refresh_token`)
- `src-tauri/src/quota_refresh_worker.rs:18-47,68-88` (`collect_all_profile_paths`, worker loop)
- `src-tauri/src/commands/quota_commands.rs:14-23,25-66` (`get_usage_limits`, `get_profile_usage`, `resolve_claude_token`)
- `src-tauri/src/commands/token_refresh.rs:20-40` (`refresh_active_token`)
- `src-tauri/src/tray.rs:148-150` (active subscription_type)
- `src-tauri/src/priming/scheduler.rs:60-68` (`creds_path_for`) + `priming/prime.rs:19-48` (`prime_account`)

## Overview
- Priority: P1. Status: pending. Depends on phase-01, phase-02.
- The 4 remaining active touchpoints all resolve a token or info from the active credential.
  On Keychain there is no path, so add **active-variant** helpers built on phase-01 blob core.

## Key insight — why not one enum for everything
Saved profiles are always files; only the *active* slot can be Keychain. YAGNI: add thin
`_active(store)` wrappers instead of threading a `CredSource` enum through every signature.
Each wrapper is ~5 lines reusing the phase-01 blob core → DRY without a broad refactor.

## Refresh design (chosen: blob-core, from phase-01)
**Option A — blob-in/blob-out core (CHOSEN).** `ensure_fresh_active`/`force_refresh_active` read the
blob from the store, run `parse_creds`+`refresh_into` (phase-01), write the blob back via
`store.write_active`. Works uniformly for file + Keychain. No temp files, no cleanup, no race window.
**Option B — temp-file bridge.** Write Keychain blob to a 0600 temp file, call existing path-based
`ensure_fresh_token`, read temp back, write to Keychain, delete temp. Rejected: plaintext secret on
disk (even briefly), cleanup-on-panic burden, extra IO — violates KISS/security for no gain now that
phase-01 already split the core.

## Architecture / data flow

### `oauth.rs` — add active variants
```
pub async fn ensure_fresh_active(store: &ActiveStore) -> Result<String, String>;
pub async fn force_refresh_active(store: &ActiveStore) -> Result<String, String>;
```
Body: `let blob = store.read_active().ok_or("no active credentials")?;`
`let mut creds = parse_creds(&blob)?;` … refresh via `refresh_into(&mut creds.root, rt)` …
`store.write_active(&serialize_root(&creds.root)?)?;` return token. Mirror the TTL/skew and
best-effort semantics of `ensure_fresh_token` (oauth.rs:98) and hard-fail of
`force_refresh_token` (oauth.rs:122).

### `token_refresh.rs::refresh_active_token` (token_refresh.rs:20-40)
- `let store = ActiveStore::resolve(&app)?;`
- `if !store.active_exists()` → same "No active credentials found" message.
- `claude_oauth::force_refresh_active(&store).await` instead of `force_refresh_token(&creds_path)`.
- `refresh_profile_token` (token_refresh.rs:47-73) UNCHANGED (profile file).

### `quota_commands.rs`
- `get_usage_limits` (quota_commands.rs:14-23): build store; token via `ensure_fresh_active(&store)`;
  fallback `parse_token(&store.read_active().unwrap_or_default())`. Return `Ok(None)` when
  `!store.active_exists()`.
- `get_profile_usage` (quota_commands.rs:25-54): the `is_active` branch → use store
  (`ensure_fresh_active`); the saved-profile branch stays path-based (`resolve_claude_token`).
  Adjust the existing `active_path` fallback (quota_commands.rs:42-44) to `store.active_exists()`.
- Keep `resolve_claude_token(path)` for profile paths; add store-based resolution inline for active.

### `quota_refresh_worker.rs`
- `collect_all_profile_paths` (quota_refresh_worker.rs:18-47) returns `(name, PathBuf)`. The active
  entry (lines 21-26) has no path on Keychain. Change return to an enum:
  `enum CredRef { Active(ActiveStore), Profile(PathBuf) }` with `name`.
- Loop (quota_refresh_worker.rs:68-88): token = match ref { Active(s) => ensure_fresh_active(&s),
  Profile(p) => ensure_fresh_token(&p) }. Everything else (usage fetch, emit, tray) unchanged.
- `idx==0` is still the active entry (pushed first) → `active_limits` logic intact.

### `tray.rs` (tray.rs:148-150)
- Replace `read_credential_info(&d.join(".credentials.json"))` with:
  `let store = ActiveStore::resolve(handle).ok();`
  `store.and_then(|s| s.read_active()).map(|b| config::parse_credential_info(&b)).and_then(|i| i.subscription_type)`.
- Saved-profile plan reads (tray.rs:173-176) UNCHANGED.

### `priming/scheduler.rs` + `prime.rs`
- `creds_path_for` (scheduler.rs:60-68) returns active path when name==active. Change `run_one`
  (scheduler.rs:51-58) to branch: active → `prime_active(store)`, profile → `prime_account(path)`.
- Add `priming/prime.rs::prime_active(store: &ActiveStore) -> PrimeResult` = same body as
  `prime_account` (prime.rs:19-48) but token via `ensure_fresh_active(&store)`. Extract the shared
  post-token logic into `prime_with_token(token)` to avoid duplication (DRY).
- `creds_path_for` becomes `active_source_for(app, name) -> Option<CredRef>` (reuse worker enum, or
  a local one). Manual `prime_now` command (calls `run_one`) inherits the fix automatically.

## Related code files
Modify:
- `src-tauri/src/modules/providers/claude_cli/oauth.rs` (add 2 active fns)
- `src-tauri/src/commands/token_refresh.rs`
- `src-tauri/src/commands/quota_commands.rs`
- `src-tauri/src/quota_refresh_worker.rs`
- `src-tauri/src/tray.rs`
- `src-tauri/src/priming/scheduler.rs`
- `src-tauri/src/priming/prime.rs`

Create/Delete: none. Consider a shared `CredRef` enum (worker + priming) in `active_store.rs`.

## Implementation steps
1. Add `ensure_fresh_active` / `force_refresh_active` to `oauth.rs`.
2. `refresh_active_token` → store + `force_refresh_active`.
3. `get_usage_limits` / `get_profile_usage` active branches → store.
4. Introduce `CredRef` enum; refactor `collect_all_profile_paths` + worker loop.
5. `tray.rs` active subscription via store.
6. `prime.rs` add `prime_active` + `prime_with_token`; `scheduler.rs` branch active vs profile.
7. `cargo build && cargo test` on Linux (File backend → identical behavior).

## Todo
- [ ] oauth active variants
- [ ] token_refresh active
- [ ] quota_commands active branches
- [ ] worker `CredRef` refactor
- [ ] tray active subscription
- [ ] priming `prime_active` + scheduler branch
- [ ] Linux build + tests green

## Success criteria
- No active-slot `.credentials.json` path reads remain outside `ActiveStore`
  (grep `\.credentials\.json` → only File backend + saved-profile joins).
- Linux behavior unchanged; on Mac (phase-04) refresh rotates the Keychain value in place.

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Worker active/profile ordering breaks `idx==0` active detection | Med | Med | Keep active pushed first; assert in test |
| Keychain write on every refresh triggers macOS auth prompt | Med | High | `-U` upsert on same app ACL should not re-prompt; verify on Mac (phase-04) |
| Refresh writes Keychain but CLI caches old token in-session | Low | Low | Same as file today; message already tells user to restart Claude |
| Duplicated prime logic drifts | Low | Low | Extract `prime_with_token` shared core |

## Security considerations
- Refresh keeps rotating refresh-token semantics (oauth.rs:78-83) — write full blob back atomically
  to the same backend that produced it.
- Never write the refreshed blob to both backends (avoid a stale plaintext file shadowing Keychain).

## Next steps
Unblocks phase-04 testing. Requires phase-01 stdin-secret verification done first.
</content>
