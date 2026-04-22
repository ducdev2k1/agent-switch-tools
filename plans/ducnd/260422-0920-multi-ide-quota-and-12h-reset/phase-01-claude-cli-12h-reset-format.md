# Phase 1: Claude CLI — Reset Time 12h Format

**Priority:** High
**Status:** DONE ✓
**Scope:** Frontend only

## Overview

Hiện tại component `UsageLimitsDisplay` chỉ show relative time như `R: 2h 15m`. User muốn thêm 12h clock format (3:45 PM) để biết chính xác khi nào reset.

## Key Insights

- Backend trả về `resets_at` dạng ISO-8601 UTC string — đã có sẵn
- Chỉ cần xử lý format phía frontend
- Dùng `Intl.DateTimeFormat` với `hour12: true` — theo timezone của máy user

## Requirements

### Functional
- Mỗi usage row (5h, 7d, 7d sonnet) hiển thị cả relative + absolute time
- Format 12h clock: `3:45 PM` (theo locale user)
- Vẫn giữ compact mode (không hiển thị absolute trong compact)

### Non-functional
- Không break existing layout (row hiện tại đã tight)
- Dùng `tabular-nums` cho clock để không jitter

## Architecture

```
UsageLimitsDisplay
  └── UsageRow
        └── formatResetsIn()   ← existing, relative
        └── formatResetsAt()   ← NEW, 12h absolute clock
```

## Related Code Files

**Modify:**
- `src/components/usage-limits-display.tsx` — thêm formatter + render

**No changes:**
- Backend (đã trả về ISO-8601)
- `src/hooks/use-ide-usage.ts`
- `src/lib/types.ts`

## Implementation Steps

1. Thêm helper `formatResetsAt(resetsAt: string | null): string | null`:
   ```tsx
   function formatResetsAt(resetsAt: string | null): string | null {
     if (!resetsAt) return null
     const d = new Date(resetsAt)
     if (isNaN(d.getTime())) return null
     return d.toLocaleTimeString(undefined, {
       hour: 'numeric',
       minute: '2-digit',
       hour12: true,
     })
   }
   ```

2. Trong `UsageRow`, tính `absText` song song với `resetText` rồi render:
   ```tsx
   const absText = formatResetsAt(bucket.resetsAt)
   // ...
   {resetText && (
     <div className="flex justify-end gap-1.5">
       <p className="text-[9px] font-medium text-muted-foreground/50 tabular-nums">
         R: {resetText.replace(/[^0-9hm]/g, '').trim()}
       </p>
       {absText && (
         <p className="text-[9px] font-medium text-muted-foreground/40 tabular-nums">
           ({absText})
         </p>
       )}
     </div>
   )}
   ```

3. Compact mode không thay đổi (giữ gọn).

4. Test manual: mở app, check Claude profile card hiển thị đúng format.

## Todo List

- [ ] Add `formatResetsAt` helper
- [ ] Render 12h clock cạnh relative text
- [ ] Verify layout không overflow trên card nhỏ
- [ ] Verify trên cả light/dark theme

## Success Criteria

- Card Claude CLI hiển thị: `R: 2h 15m (3:45 PM)` cho mỗi bucket
- Clock format dùng 12h (AM/PM)
- Compact mode không thêm clock (giữ nguyên)
- Không console error, không layout break

## Risk Assessment

- **Low**: chỉ thêm string vào row hiện có
- **Edge case**: `resets_at` invalid hoặc null → đã guard bằng `isNaN` + null check

## Next Steps

Sau phase này xong, user test visually rồi quyết định có cần tweak format không.
