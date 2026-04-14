# Phase 01: Backend — Add apiKey param to Rust webhook command

## Context
- Parent plan: [plan.md](plan.md)
- File: `src-tauri/src/commands/webhook_commands.rs`

## Overview
- Priority: P1
- Status: pending
- Add `api_key` parameter to `send_webhook` command and send via `X-API-Key` header

## Key Insights
- Current code at line 237-240 only handles `secret` → `Authorization: Bearer {secret}`
- Need parallel logic for `api_key` → `X-API-Key: {api_key}`
- Both optional, independent of each other

## Requirements
- Add `api_key: Option<String>` param to `send_webhook`
- Send `X-API-Key` header when api_key provided and non-empty
- Keep existing `Authorization: Bearer` logic unchanged
- Test mode also sends API key header

## Related Code Files
- Modify: `src-tauri/src/commands/webhook_commands.rs` (lines 173-186 signature, lines 237-240 header logic)

## Implementation Steps

1. Add `api_key: Option<String>` parameter after `secret` in `send_webhook` function signature (line 176)
2. After the existing Bearer header block (line 237-240), add:
   ```rust
   if let Some(ref k) = api_key {
       if !k.is_empty() {
           req = req.header("X-API-Key", k.as_str());
       }
   }
   ```

## Todo
- [ ] Add `api_key` param to `send_webhook` signature
- [ ] Add `X-API-Key` header logic after Bearer header block
- [ ] Verify cargo check passes

## Success Criteria
- `cargo check` passes
- `send_webhook` accepts optional `api_key` and sends `X-API-Key` header
