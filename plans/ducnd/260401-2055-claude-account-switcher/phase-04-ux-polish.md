# Phase 4: UX Polish

**Priority:** Low | **Status:** Pending | **Depends on:** Phase 1-2

## Overview

Quality-of-life improvements: system tray for quick switching, toast notifications, auto-save active profile, better profile management UX.

## Related Files

**Modify:**
- `src-tauri/src/lib.rs` — system tray setup
- `src-tauri/tauri.conf.json` — tray icon config
- `src/pages/dashboard.tsx` — improved interactions
- `src/components/profile-card.tsx` — action improvements

**Create:**
- `src-tauri/src/tray.rs` — system tray logic

## Implementation Steps

### 1. System Tray with Quick Switch

Tauri 2 system tray menu:
```
Claude Account Manager
─────────────────────
✓ Work (active)
  Personal
  Team-Project
─────────────────────
  Open Dashboard
  Quit
```

Click a profile name → switch immediately (with confirmation if Claude running).

### 2. Auto-Save Active Profile

When user first saves a profile, remember the name in metadata. On subsequent switches, auto-save current active back to its named profile — no dialog needed.

Flow: Switch to "Personal" → current "Work" auto-saved as `.credentials-Work.json` → "Personal" activated.

### 3. Toast Notifications (Already have Sonner)

Replace `alert()` and `console.error()` with Sonner toasts:
- Success: "Switched to Personal"
- Warning: "Personal token expiring in 2h"
- Error: "Switch failed: file permission denied"

### 4. Keyboard Shortcut

`Ctrl+1/2/3...` to switch to profile by position. Or `Ctrl+Shift+S` to open quick-switch popup.

### 5. Profile Card Actions Cleanup

- Remove redundant buttons for active profile
- Add "Copy credentials path" action
- Confirmation dialog for delete (replace `window.confirm`)
- Drag-and-drop reorder (stretch goal)

## Todo

- [ ] Implement system tray with profile menu
- [ ] Auto-save active profile on switch (no dialog after first save)
- [ ] Replace all `alert()`/`confirm()` with proper dialogs
- [ ] Replace `console.error` with Sonner toasts for user-visible errors
- [ ] Add keyboard shortcuts for quick switch
- [ ] Clean up profile card actions for active vs saved

## Success Criteria

- One-click switch from system tray
- No native browser dialogs (alert/confirm/prompt) remain
- Toast notifications for all user actions
- Keyboard shortcut works for top 3 profiles
