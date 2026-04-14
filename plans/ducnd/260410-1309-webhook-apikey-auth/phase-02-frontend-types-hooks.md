# Phase 02: Frontend — Types + Config Hook + Sender Hook

## Context
- Parent plan: [plan.md](plan.md)
- Depends on: Phase 01 (Rust backend must accept `api_key` param)

## Overview
- Priority: P1
- Status: pending
- Add `apiKey` field to TypeScript types and pass through hooks to Rust invoke

## Key Insights
- `WebhookConfig` interface at `src/lib/types.ts:111-119`
- Config hook at `src/hooks/use-webhook-config.ts` has DEFAULTS object (line 7-14)
- Sender hook at `src/hooks/use-webhook-sender.ts` calls `invoke('send_webhook', {...})` (line 37-44)
- UI panel at `src/components/webhook-settings-panel.tsx` also calls `invoke('send_webhook', {...})` directly for test/send (lines 95-105, 133-143)

## Requirements
- Add `apiKey: string` to `WebhookConfig` interface
- Add default `apiKey: ''` in config hook DEFAULTS
- Pass `apiKey` to `invoke('send_webhook')` in sender hook
- Pass `apiKey` to `invoke('send_webhook')` in UI panel test/send handlers

## Related Code Files
- Modify: `src/lib/types.ts` (line 114, add `apiKey: string`)
- Modify: `src/hooks/use-webhook-config.ts` (line 10, add `apiKey: ''`)
- Modify: `src/hooks/use-webhook-sender.ts` (line 39, add `apiKey: cfg.apiKey || null`)
- Modify: `src/components/webhook-settings-panel.tsx` (lines 99, 138, add `apiKey: draft.apiKey || null`)

## Implementation Steps

1. **types.ts** — Add `apiKey: string` after `secret` field:
   ```ts
   export interface WebhookConfig {
     enabled: boolean
     url: string
     secret: string
     apiKey: string          // <-- new
     triggerMode: WebhookTriggerMode
     includeCredentials: boolean
     includeSessionUsage: boolean
     memberEmail: string
   }
   ```

2. **use-webhook-config.ts** — Add to DEFAULTS:
   ```ts
   apiKey: '',
   ```

3. **use-webhook-sender.ts** — Add to invoke params (line 39):
   ```ts
   apiKey: cfg.apiKey || null,
   ```

4. **webhook-settings-panel.tsx** — Add `apiKey` to both invoke calls:
   - `handleTest` (line 99): add `apiKey: draft.apiKey || null`
   - `handleSendNow` (line 138): add `apiKey: draft.apiKey || null`

## Todo
- [ ] Add `apiKey` to WebhookConfig interface
- [ ] Add `apiKey` default in config hook
- [ ] Pass `apiKey` in sender hook invoke
- [ ] Pass `apiKey` in UI panel test/send invoke calls
- [ ] Verify `npx tsc --noEmit` passes

## Success Criteria
- TypeScript compiles without errors
- `apiKey` flows from config → hooks → Rust invoke
