---
title: "macOS Keychain support for active Claude credential"
description: "Abstract the active credential store so macOS reads/writes Claude Code creds in the login Keychain instead of a file."
status: pending
priority: P1
effort: ~14h
branch: main
tags: [tauri, rust, macos, keychain, credentials, claude-cli]
created: 2026-07-02
---

# macOS Keychain Credential Support

## Problem (verified)
Active credential I/O is hardcoded to the file `~/.claude/.credentials.json` for every OS
(`src-tauri/src/modules/shared/paths.rs:12-14` `claude_dir` + `.join(".credentials.json")`
at 9 call sites). On macOS the Claude Code CLI stores creds in the **login Keychain**, so the
file is absent. Result on default macOS: list shows no active, save errors "No active
credentials", switch writes a file the CLI ignores (fake switch), reconcile no-ops, refresh
targets nothing. Repo grep confirms 0 lines touching Keychain / `security`.

**Keychain service name (verified from a production macOS switcher, hoangpm96/ai-switcher):**
Claude Code 2.x keys each config dir's entry by a path hash —
`Claude Code-credentials-<sha256(config_dir_path)[:8 hex]>` (4 bytes → 8 hex chars). Older CLIs
used the un-suffixed global name `Claude Code-credentials`. The stored value is the identical JSON
blob (`{"claudeAiOauth":{...}}`) as the file.

## Approach decision: B (Keychain backend for active slot) — NOT A (isolated config dirs)
ai-switcher solves multi-account by giving each account its own `CLAUDE_CONFIG_DIR` (own keychain
slot) + a shell hook + dedicated `claude-<name>` commands (non-destructive pointer switch). That
is the "purest" model but is a full rewrite of this app's switch flow, changes UX, and **cannot be
tested on this Linux dev host**. We keep this app's existing model (single `~/.claude`, restart CLI
after switch) and only fix the storage layer to use the Keychain on macOS — reusing ai-switcher's
proven keychain technique. Isolated-config-dir is a possible future roadmap, not this change.

## Goal
Introduce one abstraction — `ActiveStore` — for the **active** credential only. macOS backend =
Keychain via `security` CLI (hybrid: Keychain-first, file fallback). Linux/Windows backend =
existing file, unchanged. Saved profiles under `~/.agent-switch-tools/claude/profiles/{email}/`
stay plain files — NOT changed.

## Design decisions (see phases for detail)
- **Backend selection uses runtime `cfg!(target_os = "macos")`, NOT `#[cfg]`.** All keychain
  helpers call `std::process::Command::new("security")` which compiles on every OS, so the whole
  keychain branch is type-checked by `cargo check` on Linux (only fails at runtime off-macOS,
  which never executes because `cfg!` gates it). This is the key safety property for shipping a
  macOS feature from a Linux host. `#[cfg(unix)]` is still used only for `PermissionsExt` (0o600).
- **Keychain service name:** `Claude Code-credentials-<sha256(claude_dir)[:8]>` via `sha2` (already
  a dep). Read order: per-dir hashed service → un-suffixed global name → file. Write targets the
  slot that currently holds the entry; default to the hashed service.
- **Keychain write (from ai-switcher, production-verified):**
  `security add-generic-password -U -A -s <service> -a <account> -w <blob>`, then **read back by
  service and confirm the blob matches**; **retry up to 3×** (200ms) to ride out a transient lock.
  `-U` upserts in place; `-A` grants access without a per-launch prompt (required for an unsigned
  local build). Read + write both go through the same `security` binary so macOS treats them as one
  access decision. Trade-off of `-A`/argv: acceptable on a personal machine (documented in phase-01).
- **Refresh flow:** refactor `oauth.rs` to a **blob-in/blob-out core** so refresh works for both
  file and Keychain active. Preferred over temp-file bridge (see phase-03 trade-off).
- **Keychain↔file mirror:** on a Keychain write, also mirror to the file when the file already
  exists (keeps a DarkWake-readable copy); never create a plaintext file where none existed.
- **`~/.claude.json` identity** (`oauthAccount`) stays file-based — CLI still writes it on macOS
  (ASSUMPTION, must verify on Mac — see Open Questions).

## Phases
| # | Phase | Status | Depends on |
|---|-------|--------|-----------|
| 01 | `ActiveStore` abstraction + Keychain/file backends + oauth/config blob-core refactor | ✅ done | — |
| 02 | Integrate into 3 core commands + reconcile | ✅ done | 01 |
| 03 | Integrate refresh + quota worker/commands + tray + priming (+ webhook_commands) | ✅ done | 01, 02 |
| 04 | Testing: Linux unit tests (file backend + blob core) — 26 tests pass; manual macOS guide pending on a Mac | ✅ done (Linux) | 01-03 |

## Implementation status (2026-07-02)
Implemented, `cargo check --tests` clean, 16 unit + 10 integration tests pass on Linux. Code
review surfaced one missed touchpoint — `webhook_commands.rs` (`get_profile_usage_data` active
branch + `include_credentials`) still read the active file directly — now also routed through
`ActiveStore`. **10 active touchpoints** total migrated. macOS Keychain branch is compile-checked
only (Linux dev host); the `security`-CLI paths still need one manual run against a real Claude
Code 2.x keychain entry before the macOS build ships (see Open Questions).

## Non-negotiable constraints
- Linux/Windows behavior byte-for-byte unchanged (file backend is the same code path).
- Existing `#[cfg(unix)]` 0o600 permission blocks stay for the file backend.
- macOS Keychain path must be built + tested on a real Mac (dev host is Linux).
- No new crate unless stdin-secret approach fails verification (then `security-framework`).

## Detail files
- phase-01-active-store-abstraction.md
- phase-02-core-commands-and-reconcile.md
- phase-03-refresh-quota-tray-priming.md
- phase-04-testing.md
</content>
</invoke>
