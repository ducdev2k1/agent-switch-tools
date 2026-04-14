---
title: "Phase 3: UI — Events Checkboxes + Custom Headers Editor"
status: pending
priority: P1
effort: 1.5h
---

# Phase 3: UI — Events Subscription + Custom Headers Editor

## Overview
Thay thế Trigger Mode dropdown bằng Events Subscription checkboxes. Thêm Custom Headers editor với dynamic key-value rows.

## Related Files
- `src/components/webhook-settings-panel.tsx` — main UI file

## Implementation Steps

### 1. Replace Trigger Mode dropdown với Events Subscription checkboxes

Xóa section `{/* Trigger Mode */}` (Select dropdown).
Thêm section mới:

```tsx
{/* Events Subscription */}
<div className="px-4 py-3 space-y-2">
  <Label className="text-xs text-muted-foreground">
    {t('settings.webhook.events_title')}
  </Label>
  <p className="text-[11px] text-muted-foreground">
    {t('settings.webhook.events_description')}
  </p>
  <div className="space-y-1.5 mt-2">
    {WEBHOOK_EVENTS.map((evt) => (
      <label key={evt.id} className="flex items-center gap-2 text-sm cursor-pointer">
        <Checkbox
          checked={draft.subscribedEvents.includes(evt.id)}
          onCheckedChange={(checked) => {
            const current = draft.subscribedEvents
            const next = checked
              ? [...current, evt.id]
              : current.filter((e) => e !== evt.id)
            updateField('subscribedEvents', next)
          }}
          disabled={disabled}
        />
        <span>{t(`settings.webhook.event_${evt.id}`)}</span>
      </label>
    ))}
  </div>
</div>
```

Constants:
```typescript
const WEBHOOK_EVENTS = [
  { id: 'usage_report' as const },
  { id: 'profile_switched' as const },
  { id: 'profile_created' as const },
  { id: 'profile_deleted' as const },
  { id: 'app_startup' as const },
] as const
```

### 2. Add Custom Headers editor section

Sau Events Subscription, thêm collapsible section:

```tsx
{/* Custom Headers */}
<div className="px-4 py-3 space-y-2">
  <div className="flex items-center justify-between">
    <Label className="text-xs text-muted-foreground">
      {t('settings.webhook.custom_headers')}
    </Label>
    <Button
      variant="ghost"
      size="sm"
      onClick={addHeader}
      disabled={disabled || draft.customHeaders.length >= 10}
      className="h-6 text-xs px-2"
    >
      <Plus className="size-3 mr-1" />
      {t('settings.webhook.add_header')}
    </Button>
  </div>
  <p className="text-[11px] text-muted-foreground">
    {t('settings.webhook.custom_headers_hint')}
  </p>
  {draft.customHeaders.map((header, index) => (
    <div key={index} className="flex items-center gap-2">
      <Input
        value={header.key}
        onChange={(e) => updateHeader(index, 'key', e.target.value)}
        placeholder="Header-Name"
        disabled={disabled}
        className="h-7 text-xs flex-1"
      />
      <Input
        value={header.value}
        onChange={(e) => updateHeader(index, 'value', e.target.value)}
        placeholder="value"
        disabled={disabled}
        className="h-7 text-xs flex-1"
      />
      <button
        type="button"
        onClick={() => removeHeader(index)}
        disabled={disabled}
        className="text-muted-foreground hover:text-destructive p-1 cursor-pointer"
      >
        <X className="size-3.5" />
      </button>
    </div>
  ))}
</div>
```

### 3. Add helper functions trong component

```typescript
const addHeader = () => {
  if (draft.customHeaders.length >= 10) return
  updateField('customHeaders', [...draft.customHeaders, { key: '', value: '' }])
}

const removeHeader = (index: number) => {
  updateField('customHeaders', draft.customHeaders.filter((_, i) => i !== index))
}

const updateHeader = (index: number, field: 'key' | 'value', value: string) => {
  const updated = draft.customHeaders.map((h, i) =>
    i === index ? { ...h, [field]: value } : h
  )
  updateField('customHeaders', updated)
}
```

### 4. Import thêm components

```typescript
import { Checkbox } from '@/components/ui/checkbox'
// Add to lucide imports: Plus, X
```

Kiểm tra Checkbox component đã có trong project chưa. Nếu chưa: `pnpm dlx shadcn@latest add checkbox`.

### 5. Cleanup
- Xóa import `Select, SelectContent, SelectItem, SelectTrigger, SelectValue`
- Xóa import `WebhookTriggerMode` từ types
- Xóa toàn bộ Trigger Mode section

## Todo
- [ ] Kiểm tra/thêm Checkbox shadcn component
- [ ] Replace Trigger Mode dropdown với Events checkboxes
- [ ] Add Custom Headers dynamic editor (add/remove/edit rows)
- [ ] Add helper functions (addHeader, removeHeader, updateHeader)
- [ ] Cleanup unused imports
- [ ] Update handleSave validation (nếu enabled, cần ít nhất 1 event hoặc manual)

## Success Criteria
- Events checkboxes render đúng, toggle works
- Custom headers add/remove/edit works smoothly
- Max 10 headers limit enforced
- UI disabled khi webhook off
- Responsive trên cả desktop và mobile
