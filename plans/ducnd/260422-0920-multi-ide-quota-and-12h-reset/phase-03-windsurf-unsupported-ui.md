# Phase 3: Windsurf — Graceful "Not Supported"

**Priority:** Low
**Status:** DONE ✓ (extended to Cursor too since quota also unsupported)
**Scope:** Frontend (backend không cần thay đổi vì đã return None)

## Overview

Windsurf không có single-user quota API (chỉ Enterprise API + service key). Thay vì để card loading mãi hoặc trống, hiển thị rõ "Quota không khả dụng" kèm tooltip giải thích.

## Key Insights

- Backend `get_ide_usage` cho Windsurf sẽ return `None` vì không có quota logic
- UI hiện tại khi `limits = null` → không render gì (usage-limits-display.tsx:104)
- Cần phân biệt "chưa fetch" vs "API không support"

## Requirements

### Functional
- Windsurf profile card có placeholder text `Quota không khả dụng cho Windsurf` (hoặc tooltip) thay vì trống
- Các IDE khác giữ behavior cũ (không render khi null)

### Non-functional
- Nhỏ gọn, không chiếm quá nhiều không gian
- Translation key để support i18n

## Architecture

Hai approach:

**Option A** (đơn giản): Prop `unsupported?: boolean` trên `UsageLimitsDisplay`, pass từ parent khi `ideType === 'windsurf'`.

**Option B** (tự động): Backend return `UsageLimits { unsupported: true }` flag, frontend check. Cần modify backend schema.

→ Chọn **Option A** (KISS, không đụng backend).

## Related Code Files

**Modify:**
- `src/components/usage-limits-display.tsx` — thêm prop + render branch
- `src/components/ide-profile-card.tsx` + `ide-profile-table.tsx` — pass prop khi Windsurf
- `src/locales/*.json` (hoặc wherever translations) — thêm key `common.labels.usage_unsupported`

**No changes:**
- Backend (đã return None)
- Hook `use-ide-usage.ts`

## Implementation Steps

1. **Add prop** vào `UsageLimitsDisplayProps`:
   ```ts
   interface UsageLimitsDisplayProps {
     limits: UsageLimits | null
     loading?: boolean
     compact?: boolean
     unsupported?: boolean  // NEW
   }
   ```

2. **Render branch** trong component:
   ```tsx
   if (unsupported) {
     return (
       <div className="mt-4 px-0.5">
         <p className="text-[10px] font-medium text-muted-foreground/60 italic">
           {t('common.labels.usage_unsupported')}
         </p>
       </div>
     )
   }
   ```
   Phải đặt trước check `if (!limits) return null`.

3. **Pass từ parent** — trong `ide-profile-card.tsx`:
   ```tsx
   <UsageLimitsDisplay
     limits={usage}
     loading={loading}
     unsupported={ideType === 'windsurf'}
   />
   ```

4. **Grep các parent khác** (`ide-profile-table.tsx`, `ide-dashboard-section.tsx`) và wire tương tự.

5. **Add translation keys** (tìm file hiện có, thêm):
   - vi: `"usage_unsupported": "Quota không khả dụng"`
   - en: `"usage_unsupported": "Quota not available"`

6. **Test manual**: card Windsurf hiển thị message; Cursor/Claude/Antigravity không đổi.

## Todo List

- [ ] Add `unsupported` prop
- [ ] Render branch khi unsupported
- [ ] Pass prop từ all parents (grep các usage)
- [ ] Add translation keys
- [ ] Manual test trên cả 4 IDE types

## Success Criteria

- Windsurf card: show "Quota không khả dụng" (italic, nhạt)
- Cursor/Claude/Antigravity: behavior không đổi
- i18n work đúng cho cả vi và en

## Risk Assessment

- **Low**: thuần UI, không đụng logic
- **Edge case**: nếu trong tương lai Windsurf public user quota API → chỉ cần xóa prop

## Next Steps

- Monitor Windsurf changelog; nếu họ public API sẽ add provider tương tự Cursor
