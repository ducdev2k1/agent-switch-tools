# Phase 6: Frontend — Dashboard Polish + System Tray

**Priority:** Medium
**Status:** Pending
**Depends on:** Phase 5

---

## Overview

Cap nhat system tray menu de hien thi IDE profiles va cho phep quick-switch tu tray. Polish dashboard UX.

## UX Design — System Tray

```
┌────────────────────────────┐
│ Claude Tools               │
├────────────────────────────┤
│ 🔹 Claude Code             │
│   ✓ user1@gmail.com        │
│     user2@company.com      │
├────────────────────────────┤
│ 🔹 Cursor                  │
│   ✓ ducnd@inet.vn          │
│     other@gmail.com        │
├────────────────────────────┤
│ 🔹 Antigravity             │
│   ✓ ducdev2k1@gmail.com    │
├────────────────────────────┤
│ Open Dashboard              │
│ Quit                        │
└────────────────────────────┘
```

## Related Code Files

**Modify:**
- `src-tauri/src/tray.rs` — add IDE sections to tray menu

**Read for context:**
- `src-tauri/src/tray.rs` — current tray logic
- `src-tauri/src/commands/ide_profile_commands.rs` (Phase 3)

## Implementation Steps

1. Modify `tray.rs` — `build_tray_menu()`:
   - Keep existing Claude Code section (UNCHANGED logic)
   - For each installed IDE:
     - Add separator + IDE header
     - Show active profile with checkmark
     - List saved profiles as switch items
   - Event IDs: `ide-switch:{ideType}:{profileName}`

2. Handle tray events:
   - `ide-switch:cursor:user@gmail.com` → emit `tray-switch-ide-profile` event
   - Frontend receives event → show confirmation dialog

3. Modify `refresh_tray_menu()`:
   - Also refresh IDE sections when IDE profiles change

4. Dashboard polish:
   - Add empty state for IDEs with no saved profiles
   - Add "IDE not installed" indicator for missing IDEs
   - Smooth tab transitions

## Todo

- [ ] Update `build_tray_menu()` with IDE sections
- [ ] Handle tray IDE switch events
- [ ] Update `refresh_tray_menu()` for IDEs
- [ ] Polish dashboard tab transitions
- [ ] Test tray menu on Linux

## Success Criteria

- Tray shows all IDEs with their profiles
- Quick-switch from tray works for IDE profiles
- Dashboard looks polished with tab selector
- Tray refreshes when IDE profiles change
