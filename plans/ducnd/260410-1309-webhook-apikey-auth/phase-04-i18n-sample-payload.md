# Phase 04: i18n + Sample Payload Update

## Context
- Parent plan: [plan.md](plan.md)
- Depends on: Phase 03 (UI needs i18n keys)

## Overview
- Priority: P2
- Status: pending
- Add i18n strings for API Key field, update sample payload comment

## Related Code Files
- Modify: `src/locales/en.json` (webhook section, around line 139)
- Modify: `src/components/webhook-settings-panel.tsx` (buildSamplePayload function, line 384+)

## Implementation Steps

1. **en.json** — Add after `secret_placeholder` (line 140):
   ```json
   "api_key": "API Key",
   "api_key_placeholder": "Your API key (optional)",
   "api_key_hint": "Sent via X-API-Key header for service authentication",
   ```

2. **webhook-settings-panel.tsx** — No change needed to sample payload (payload structure unchanged, API key is a header not payload field)

## Todo
- [ ] Add 3 i18n keys to en.json
- [ ] Verify no missing translation warnings

## Success Criteria
- All i18n keys resolve correctly
- No console warnings about missing translations
