# Phase 05: Compile Check + Test

## Context
- Parent plan: [plan.md](plan.md)
- Depends on: Phase 01-04 all complete

## Overview
- Priority: P1
- Status: pending
- Verify both Rust and TypeScript compile, then manual test with target URL

## Implementation Steps

1. Run `cargo check` in `src-tauri/` — verify no Rust errors
2. Run `pnpm tsc --noEmit` — verify no TypeScript errors
3. Run `pnpm build` — verify full build passes
4. Manual test: set URL to `https://claude.inetdev.io.vn/api/webhook/usage-report` with API key, click Test Connection

## Todo
- [ ] cargo check passes
- [ ] TypeScript check passes
- [ ] Full build passes

## Success Criteria
- Zero compile errors on both Rust and TypeScript
- Webhook sends with `X-API-Key` header when API key configured
