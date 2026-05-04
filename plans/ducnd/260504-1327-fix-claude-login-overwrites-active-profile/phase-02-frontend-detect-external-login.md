# Phase 2 — Frontend: Auto-refresh on focus + drift toast

**Status:** Pending
**Priority:** High
**Effort:** Small
**Depends on:** Phase 1

## Context Links

- Backend event: `claude-profile-drift-detected` (emitted by `reconcile_active_profile`)
- Frontend hook: [src/hooks/use-profiles.ts](../../../src/hooks/use-profiles.ts)

## Overview

User mở app sau khi login Claude ngoài app → frontend phải tự refresh profile list để hiển thị account mới. Nếu phát hiện drift, show toast thông báo "Detected external login: <email> — saved as new profile".

## Requirements

### Functional
- Khi window được focus (user quay lại app sau khi login ngoài), auto trigger `load()` profiles
- Listen event `claude-profile-drift-detected` từ backend → show toast
- Toast i18n: cả English và Vietnamese

### Non-functional
- Debounce window focus refresh (≥1s) để tránh spam khi user switch tab nhanh
- Không block UI khi load

## Architecture

```
window.focus  ──► debounced refresh ──► list_credential_profiles ──► backend reconcile
                                                                       │
                                                                       └─► emit event
backend event ──► useEffect listen  ──► toast notification
```

## Related Code Files

**Modify:**
- `src/hooks/use-profiles.ts` — thêm focus listener + event listener
- `src/locales/en.json` — thêm i18n key `profiles.driftDetected`
- `src/locales/vi.json` — thêm i18n key `profiles.driftDetected`

**Read for context:**
- `src/App.tsx` — xem toast lib đang dùng (sonner / react-hot-toast / shadcn)
- `src/lib/types.ts` — type `CredentialProfile`

## Implementation Steps

### Step 1: Xác định toast lib

```bash
grep -rn "toast\|Toaster" src/App.tsx src/main.tsx 2>/dev/null
grep -rn "sonner\|react-hot-toast\|Toaster" src/components 2>/dev/null | head -5
cat package.json | grep -i toast
```

### Step 2: Update `use-profiles.ts`

Thêm imports:
```typescript
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { toast } from 'sonner' // hoặc lib đang dùng
import { useTranslation } from 'react-i18next' // nếu app dùng
```

Trong hook, thêm 2 useEffect:

```typescript
// Auto-refresh khi window focus
useEffect(() => {
  let lastRefresh = 0
  const onFocus = () => {
    const now = Date.now()
    if (now - lastRefresh < 1000) return // debounce 1s
    lastRefresh = now
    load()
  }
  window.addEventListener('focus', onFocus)
  return () => window.removeEventListener('focus', onFocus)
}, [load])

// Listen drift event
useEffect(() => {
  let unlisten: UnlistenFn | null = null
  listen('claude-profile-drift-detected', () => {
    toast.info(t('profiles.driftDetected'))
    load()
  }).then((fn) => { unlisten = fn })
  return () => { unlisten?.() }
}, [load, t])
```

### Step 3: Update i18n keys

`src/locales/en.json`:
```json
{
  "profiles": {
    "driftDetected": "Detected new Claude login — saved previous account, switched to new one."
  }
}
```

`src/locales/vi.json`:
```json
{
  "profiles": {
    "driftDetected": "Phát hiện đăng nhập Claude mới — đã lưu tài khoản cũ, chuyển sang tài khoản mới."
  }
}
```

### Step 4: Manual test

1. `pnpm tauri dev`
2. Login account A trong CLI: `claude /login` → chọn account A
3. Mở app → bấm "Save Current as Profile" (lưu A)
4. Đóng app
5. Login account B trong CLI: `claude /login` → chọn account B
6. Mở lại app → kiểm tra:
   - Toast hiện "Detected new Claude login..."
   - Sidebar có cả A và B
   - B là active
7. Switch sang A → kiểm tra credentials A được restore, B được lưu folder `profiles/email_b@gmail.com/`

## Todo List

- [ ] Identify toast library trong project
- [ ] Update `use-profiles.ts` thêm focus listener (debounced)
- [ ] Update `use-profiles.ts` thêm event listener cho `claude-profile-drift-detected`
- [ ] Add i18n keys `profiles.driftDetected` cho en + vi
- [ ] Manual test scenario A→B switch
- [ ] Manual test rapid focus toggling không spam refresh
- [ ] `pnpm tauri dev` không có console errors

## Success Criteria

- Window focus → profiles tự refresh trong vòng <500ms
- Toast hiện đúng ngôn ngữ user đã chọn (en/vi)
- Sau external login + mở app: cả 2 profile đều hiển thị, active là profile mới

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Toast spam khi user switch trong app (drift event không nên bắn) | Drift chỉ trigger khi reconcile phát hiện meta ≠ actual; switch trong app update meta đồng bộ → no drift |
| Event listener leak | Clean up via UnlistenFn trong return của useEffect |
| Window focus event không fire trên một số platform (Wayland?) | Backup: poll mỗi 30s khi window visible — không cần thiết phase này |

## Security Considerations

- Event payload là `()` — không leak credentials
- Toast message không show email cụ thể (tránh shoulder-surfing) — chỉ message generic

## Next Steps

→ Phase 3: Tests regression
