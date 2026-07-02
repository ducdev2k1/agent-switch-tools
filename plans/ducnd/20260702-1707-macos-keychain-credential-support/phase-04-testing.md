# Phase 04 — Testing (Linux unit tests + manual macOS Keychain guide)

## Context links
- `src-tauri/src/modules/providers/claude_cli/reconcile.rs:132-227` (existing test harness pattern)
- `src-tauri/Cargo.toml:[dev-dependencies]` (`tempfile = "3"` available)
- phase-01/02/03 (units under test)

## Overview
- Priority: P1. Status: pending. Depends on phase-01..03.
- Two tracks: (A) Linux-runnable unit tests for the **File backend + blob cores** — the parts that
  are OS-neutral; (B) a manual verification script for the **macOS Keychain** backend, which cannot
  run on the Linux dev host and MUST be executed on a real Mac.

## Track A — Linux automated tests (`cargo test`)

### A1. `ActiveStore::File` round-trip (`active_store.rs` `#[cfg(test)]`)
- `write_active(blob)` then `read_active()` == blob.
- `active_exists()` false before write, true after.
- file created with 0o600 (`#[cfg(unix)]` assert mode & 0o777 == 0o600).
- `delete_active()` removes file; `active_exists()` false; `read_active()` None.
- `read_active()` on missing file → None (no panic).

### A2. Blob-core parity (no HTTP)
- `config::parse_credential_info(blob)` == `read_credential_info(tmpfile(blob))` for: valid blob,
  expired `expiresAt`, missing `claudeAiOauth`, malformed JSON (both default).
- `quota_commands::parse_token(blob)` == `read_token_from_creds(tmpfile(blob))`.
- `oauth::parse_creds(blob)` extracts access/refresh/expiresAt; missing accessToken → Err.

### A3. Reconcile via `ActiveStore::file` (adapt reconcile.rs:174-226)
- Rework the 2 existing tests to pass `ActiveStore::file(claude.join(".credentials.json"))`.
- Assert identical outcomes (drift detect + backup preserve; matching-email refresh). Proves phase-02
  signature change is behavior-neutral.

### A4. Refresh core (mock-free, logic only)
- `needs_refresh` boundary (oauth.rs:131): expired, within-skew, fresh, `None`.
- (HTTP `refresh_into` not unit-tested; covered by manual Mac test + existing manual flows.)

Run: `cd src-tauri && cargo test`. All A1-A4 + prior suites green on Linux.

## Track B — macOS manual Keychain verification (real Mac required)
Prereqs: build `cargo check` / `cargo build` on macOS (validates `cfg(macos)` Keychain arm compiles).

### B0. Verify the stdin-secret assumption FIRST (blocks phase-01 sign-off)
```
SECRET='{"claudeAiOauth":{"accessToken":"t","refreshToken":"r","expiresAt":0,"scopes":[]}}'
printf '%s' "$SECRET" | security add-generic-password -s "Claude Code-credentials" -a "$USER" -U -w
security find-generic-password -s "Claude Code-credentials" -a "$USER" -w   # must print SECRET
```
If `-w` does NOT read from stdin → switch to `security-framework` crate (update phase-01).
Also confirm no plaintext secret appears in `ps aux | grep security` during the write.

### B1. Existing state discovery
- `security find-generic-password -s "Claude Code-credentials" -a "$USER" -w` — capture current CLI blob.
- Confirm `~/.claude.json` still contains `oauthAccount.emailAddress` (identity assumption).

### B2. list — active shows up
- Launch app → active profile appears with correct email/plan (was empty pre-fix).

### B3. save_current — profile file written
- Click save → `~/.agent-switch-tools/claude/profiles/{email}/credentials.json` exists, 0600,
  blob == Keychain value.

### B4. switch — Keychain actually updated
- Switch to profile B → `security find-generic-password ... -w` now returns B's blob.
- Run `claude` (or check CLI) → CLI sees account B (proves real switch, not fake file).
- Restore original.

### B5. reconcile — external login
- `claude /login` to a new account outside the app → open app/list → drift detected, new account
  snapshotted, old profile preserved (mirror A3 on real Keychain).

### B6. refresh — in-place Keychain rotation
- Manual "Refresh Token" → success; `security find-generic-password ... -w` shows a new
  `accessToken`/`expiresAt`; confirm NO repeated Keychain auth prompt (ACL check).

### B7. quota + tray + priming
- Tray shows active plan/usage %; quota worker updates without error (check logs);
  trigger prime on active → succeeds using Keychain token.

### B8. Hybrid file-mode fallback
- Delete Keychain entry, create `~/.claude/.credentials.json` file → app still lists/switches
  (fallback to file); switch writes to file (not Keychain) per hybrid policy.

## Todo
- [ ] A1 File backend tests
- [ ] A2 blob-core parity tests
- [ ] A3 reconcile via `ActiveStore::file`
- [ ] A4 `needs_refresh` boundary
- [ ] `cargo test` green on Linux
- [ ] B0 stdin-secret verified on Mac (gate)
- [ ] B1-B8 manual Mac checklist executed + logged

## Success criteria
- Track A fully green on Linux CI.
- Track B checklist passes on a real Mac; B0 confirms the `security` stdin approach (or triggers the
  documented `security-framework` fallback).

## Risk assessment
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| No Mac available to run Track B before release | Med | High | Do NOT ship macOS build until B0-B7 pass; keep feature `cfg`-guarded |
| Keychain ACL prompts on every access annoy users | Med | Med | Verify B6; if prompts, evaluate `security-framework` with explicit ACL |
| `~/.claude.json` lacks email on some Mac setups | Low | High | B1 gate; if missing, plan a follow-up to derive identity from blob |

## Security considerations
- Test blobs use fake tokens only; never commit real credentials.
- Manual steps read secrets to terminal — run on a trusted machine, clear scrollback after.

## Next steps
On green: update `docs/` (system-architecture + changelog) noting macOS Keychain active-store.
</content>
