# Multi-IDE Account Switching

**Created:** 2026-04-16
**Status:** Draft
**Priority:** High
**Branch:** `feat/multi-ide-account-switching`

---

## Overview

Nang cap Claude Tools de ho tro switch account cho nhieu loai IDE: **Cursor** va **Antigravity** (Windsurf). Giu nguyen logic Claude Code hien tai, them layer IDE abstraction cho 2 IDE moi.

## Research Summary

| IDE | Credential Storage | DB Path (Linux) | Key Auth Fields |
|---|---|---|---|
| Claude Code | Plaintext JSON | `~/.claude/.credentials.json` | `claudeAiOauth` object |
| Cursor | SQLite `state.vscdb` | `~/.config/Cursor/User/globalStorage/state.vscdb` | `cursorAuth/accessToken`, `cursorAuth/refreshToken`, `cursorAuth/cachedEmail`, `cursorAuth/stripeMembershipType` |
| Antigravity | SQLite `state.vscdb` | `~/.config/Antigravity/User/globalStorage/state.vscdb` | `antigravityAuthStatus` (JSON: name, apiKey, email, userStatusProtoBinaryBase64), `antigravityUnifiedStateSync.oauthToken` |

### Cross-platform paths

| IDE | Linux | macOS | Windows |
|---|---|---|---|
| Cursor | `~/.config/Cursor/User/globalStorage/` | `~/Library/Application Support/Cursor/User/globalStorage/` | `%APPDATA%\Cursor\User\globalStorage\` |
| Antigravity | `~/.config/Antigravity/User/globalStorage/` | `~/Library/Application Support/Antigravity/User/globalStorage/` | `%APPDATA%\Antigravity\User\globalStorage\` |

### Approach: Selective Key Backup/Restore

Thay vi backup toan bo `state.vscdb` (300MB+), chi backup/restore cac auth keys tu SQLite.

- **Cursor**: Read/write 5 keys (`cursorAuth/*`) tu `ItemTable`
- **Antigravity**: Read/write 2 keys (`antigravityAuthStatus`, `antigravityUnifiedStateSync.oauthToken`) tu `ItemTable`

Luu backup dang JSON file nhe (< 5KB/profile), tuong tu cach Claude Code luu `credentials.json`.

## Architecture

```
~/.claude/.claude-tools/
  meta.json                    (global metadata)
  profiles/                    (Claude Code profiles - EXISTING, UNCHANGED)
    user1@gmail.com/
      credentials.json
      oauth.json
  cursor/                      (NEW: Cursor profiles)
    profiles/
      user1@gmail.com/
        auth-keys.json         (cursorAuth/* keys backup)
    meta.json                  (active profile, usage history)
  antigravity/                 (NEW: Antigravity profiles)
    profiles/
      user2@gmail.com/
        auth-keys.json         (antigravity auth keys backup)
    meta.json
```

## Phases

| # | Phase | Status | Est. Effort |
|---|-------|--------|-------------|
| 1 | [Backend: IDE registry + path resolver](phase-01-backend-ide-registry.md) | Pending | Medium |
| 2 | [Backend: SQLite credential read/write](phase-02-backend-sqlite-credentials.md) | Pending | Medium |
| 3 | [Backend: Profile CRUD per IDE](phase-03-backend-profile-crud.md) | Pending | Medium |
| 4 | [Frontend: Types + IDE context](phase-04-frontend-types-ide-context.md) | Pending | Small |
| 5 | [Frontend: IDE selector UI](phase-05-frontend-ide-selector-ui.md) | Pending | Medium |
| 6 | [Frontend: Dashboard + tray per IDE](phase-06-frontend-dashboard-tray.md) | Pending | Medium |
| 7 | [i18n + compile + test](phase-07-i18n-compile-test.md) | Pending | Small |

## Key Decisions

1. **Claude Code logic UNCHANGED** — All existing commands/hooks/UI keep working as-is
2. **Selective key backup** — Only auth keys from SQLite, not entire DB
3. **Shared storage root** — `~/.claude/.claude-tools/{ide}/` keeps everything centralized
4. **IDE detection** — Check if DB file exists to auto-detect installed IDEs
5. **Rust SQLite** — Add `rusqlite` crate for reading/writing `state.vscdb`

## Dependencies

- `rusqlite` crate (SQLite bindings for Rust)
- No changes to existing Claude Code credential flow

## Risk Assessment

- **Cursor/Antigravity updates may change auth key names** — Mitigate: define key names as config, easy to update
- **IDE running during switch** — Mitigate: check process, warn user to close IDE first
- **state.vscdb locked by running IDE** — Mitigate: detect + warn; SQLite WAL mode should allow reads
