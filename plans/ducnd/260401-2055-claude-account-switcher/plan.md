# Claude Account Switcher - Enhancement Plan

**Date:** 2026-04-01
**Status:** Draft
**Branch:** main

## Goal

Improve credential switching tool so user can seamlessly swap Claude accounts when quota exhausted, preserving working session.

## Key Insight

- `~/.claude/.credentials.json` = active credential (OAuth tokens)
- `~/.claude/projects/`, `sessions/`, `settings.json` = session data (INDEPENDENT of credentials)
- Swapping `.credentials.json` works for next CLI session; running instance caches tokens in memory
- So: swap file -> restart/new Claude Code session -> session context preserved (project settings, history intact)

## Current State

Working: profile CRUD (list, save, switch, rename, delete), CLI state monitoring, dashboard UI.

Gaps: no active profile name tracking, no token validation, no running-process detection, `window.prompt` hack, dead code, no atomic switch, no quota visibility.

## Phases

| # | Phase | Priority | Status |
|---|-------|----------|--------|
| 1 | [Core Switch Fixes](phase-01-core-switch-fixes.md) | High | Pending |
| 2 | [Session-Safe Switching](phase-02-session-safe-switching.md) | High | Pending |
| 3 | [Quota & Usage Monitoring](phase-03-quota-monitoring.md) | Medium | Pending |
| 4 | [UX Polish](phase-04-ux-polish.md) | Low | Pending |

## Dependencies

- Phase 2 depends on Phase 1 (fixed switch logic)
- Phase 3 independent (can parallel with Phase 2)
- Phase 4 depends on Phase 1-2

## Tech Stack

Tauri 2 + React 19 + TypeScript 5.8 + Rust + Tailwind 4 + shadcn/ui
