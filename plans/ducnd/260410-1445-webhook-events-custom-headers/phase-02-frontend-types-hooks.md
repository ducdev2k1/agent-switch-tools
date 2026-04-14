---
title: "Phase 2: Frontend — Types + Config Hook + Sender Hook"
status: pending
priority: P1
effort: 1h
---

# Phase 2: Frontend — Update types, config hook, sender hook

## Overview
Update TypeScript types, config defaults, và sender hook để support events subscription + custom headers.

## Related Files
- `src/lib/types.ts` — WebhookConfig type
- `src/hooks/use-webhook-config.ts` — config storage + defaults
- `src/hooks/use-webhook-sender.ts` — trigger logic + invoke calls

## Implementation Steps

### 1. Update types (`src/lib/types.ts`)

```typescript
export interface CustomHeader {
  key: string
  value: string
}

// Available webhook events
export type WebhookEvent =
  | 'usage_report'
  | 'profile_switched'
  | 'profile_created'
  | 'profile_deleted'
  | 'app_startup'

export interface WebhookConfig {
  enabled: boolean
  url: string
  secret: string
  apiKey: string
  subscribedEvents: WebhookEvent[]     // NEW: replaces triggerMode
  customHeaders: CustomHeader[]         // NEW
  includeCredentials: boolean
  includeSessionUsage: boolean
  memberEmail: string
  // DEPRECATED — kept for migration only
  triggerMode?: WebhookTriggerMode
}
```

Giữ `WebhookTriggerMode` export cho backward compat nhưng không dùng trong UI mới.

### 2. Update config hook (`src/hooks/use-webhook-config.ts`)

Update DEFAULTS:
```typescript
const DEFAULTS: WebhookConfig = {
  enabled: false,
  url: '',
  secret: '',
  apiKey: '',
  subscribedEvents: ['usage_report'],  // default: chỉ báo cáo usage
  customHeaders: [],
  includeCredentials: false,
  includeSessionUsage: true,
  memberEmail: '',
}
```

Thêm migration logic trong `useEffect` load:
```typescript
// Migrate triggerMode → subscribedEvents
if (val.triggerMode && !val.subscribedEvents) {
  const migrated: WebhookEvent[] = []
  if (val.triggerMode === 'on_startup') migrated.push('app_startup')
  if (val.triggerMode === 'on_change') migrated.push('usage_report')
  // 'manual' → empty (user triggers manually)
  val.subscribedEvents = migrated.length > 0 ? migrated : []
  delete val.triggerMode
}
if (!val.customHeaders) val.customHeaders = []
```

### 3. Update sender hook (`src/hooks/use-webhook-sender.ts`)

Refactor trigger logic:
- Xóa `triggerMode` checks
- Listen cho nhiều events, check `subscribedEvents` array

```typescript
// Trigger: app_startup
useEffect(() => {
  if (startupFiredRef.current) return
  startupFiredRef.current = true
  const timer = setTimeout(async () => {
    const cfg = await readConfig()
    if (cfg?.enabled && cfg.subscribedEvents?.includes('app_startup') && cfg.url) {
      sendFromStore(false, 'app_startup')
    }
  }, STARTUP_DELAY_MS)
  return () => clearTimeout(timer)
}, [sendFromStore])

// Trigger: usage_report (from quota refresh)
useEffect(() => {
  const unlisten = listen('usage-updated', async () => {
    const cfg = await readConfig()
    if (!cfg?.enabled || !cfg.subscribedEvents?.includes('usage_report') || !cfg.url) return
    // cooldown check...
    sendFromStore(false, 'usage_report')
  })
  return () => { unlisten.then((fn) => fn()) }
}, [sendFromStore])
```

Update `sendFromStore` to pass `eventType` + `customHeaders`:
```typescript
const sendFromStore = useCallback(
  async (testMode = false, eventType = 'usage_report'): Promise<WebhookResponse | null> => {
    const cfg = await readConfig()
    if (!cfg || !cfg.enabled || !cfg.url) return null
    return await invoke<WebhookResponse>('send_webhook', {
      url: cfg.url,
      secret: cfg.secret || null,
      apiKey: cfg.apiKey || null,
      eventType,
      customHeaders: cfg.customHeaders?.filter(h => h.key.trim()) || null,
      testMode,
      includeCredentials: cfg.includeCredentials,
      includeSessionUsage: cfg.includeSessionUsage ?? true,
      memberEmail: cfg.memberEmail || null,
    })
  },
  [],
)
```

Thêm listeners cho profile events:
```typescript
// Trigger: profile_switched
useEffect(() => {
  const unlisten = listen('tray-switch-profile', async () => {
    const cfg = await readConfig()
    if (!cfg?.enabled || !cfg.subscribedEvents?.includes('profile_switched') || !cfg.url) return
    sendFromStore(false, 'profile_switched')
  })
  return () => { unlisten.then(fn => fn()) }
}, [sendFromStore])
```

**Lưu ý:** `profile_created` và `profile_deleted` cần emit events mới từ frontend (invoke xong → trigger). Sẽ handle bằng cách gọi `sendFromStore` trực tiếp sau invoke thành công trong `use-profiles.ts`, hoặc emit custom event từ hooks.

### 4. Expose sendForEvent từ useWebhookSender

```typescript
return {
  sendManual: sendWithCooldown,
  sendForEvent: (eventType: WebhookEvent) => sendFromStore(false, eventType),
  testConnection: () => sendFromStore(true, 'test'),
}
```

Frontend code (use-profiles.ts) gọi `sendForEvent('profile_created')` sau khi save profile thành công.

## Todo
- [ ] Add `CustomHeader` interface + `WebhookEvent` type to types.ts
- [ ] Update `WebhookConfig` interface (subscribedEvents, customHeaders)
- [ ] Update DEFAULTS in use-webhook-config.ts
- [ ] Add triggerMode → subscribedEvents migration
- [ ] Refactor use-webhook-sender.ts: multi-event listeners
- [ ] Pass eventType + customHeaders to invoke calls
- [ ] Expose sendForEvent for profile hooks integration

## Success Criteria
- `tsc --noEmit` passes
- Migration: old configs with `triggerMode` auto-convert to `subscribedEvents`
- Sender hook correctly filters by subscribed events
