# Phase 5: Frontend — IDE Selector UI

**Priority:** High
**Status:** Pending
**Depends on:** Phase 4

---

## Overview

Them IDE selector vao dashboard header de user chon IDE nao muon manage. Khi chon IDE, dashboard hien thi profiles cua IDE do.

## UX Design

### Dashboard Header
Them tab/selector ngay duoi header bar:

```
┌─────────────────────────────────────────────┐
│  🛡 Claude Tools               [↻] [💾] [⚙] │
├─────────────────────────────────────────────┤
│  [Claude Code ✓]  [Cursor]  [Antigravity]   │
├─────────────────────────────────────────────┤
│  Active Account                              │
│  ┌─────────────────────────────────────┐    │
│  │ user@gmail.com (Pro) ✓ Active       │    │
│  └─────────────────────────────────────┘    │
│  Saved Profiles (2)                          │
│  ...                                         │
└─────────────────────────────────────────────┘
```

- Claude Code tab: hien thi dashboard hien tai (KHONG THAY DOI)
- Cursor/Antigravity tabs: chi hien khi IDE installed
- Active tab luu vao settings store (persist across sessions)
- Moi tab co profile cards tuong tu Claude Code

### Profile Card cho IDE
Tuong tu ProfileCard hien tai nhung simplified:
- Email / display name
- Membership type (Cursor) hoac plan name (Antigravity)
- Active indicator
- Switch / Delete buttons

## Related Code Files

**Modify:**
- `src/pages/dashboard.tsx` — add IDE tab selector, conditional rendering
- `src/components/profile-card.tsx` — make reusable for IDE profiles (optional: create separate IdeProfileCard)

**Create:**
- `src/components/ide-selector-tabs.tsx` — tab component for IDE selection
- `src/components/ide-profile-card.tsx` — profile card for IDE accounts
- `src/components/ide-dashboard-section.tsx` — profile list + actions cho IDE

**Read for context:**
- `src/pages/dashboard.tsx` — current layout
- `src/components/profile-card.tsx` — current card design

## Implementation Steps

1. Create `ide-selector-tabs.tsx`:
   - Tabs: "Claude Code" (default), "Cursor", "Antigravity"
   - Only show installed IDEs
   - Persist selected tab to settings store
   - Emit `onIdeChange(ideType)` callback

2. Create `ide-profile-card.tsx`:
   - Simpler than ProfileCard (no quota, no expiry — IDE credentials don't expose this)
   - Show: email, membership/plan, active status
   - Actions: Switch, Rename, Delete

3. Create `ide-dashboard-section.tsx`:
   - Uses `useIdeProfiles(ideType)` hook
   - Active profile section + saved profiles list
   - Save current button + Add account dialog
   - Switch confirmation dialog (check if IDE running)

4. Modify `dashboard.tsx`:
   - Add IdeSelectorTabs at top
   - When "Claude Code" selected: show existing dashboard (UNCHANGED)
   - When IDE selected: show IdeDashboardSection

## Todo

- [ ] Create `ide-selector-tabs.tsx`
- [ ] Create `ide-profile-card.tsx`
- [ ] Create `ide-dashboard-section.tsx`
- [ ] Modify `dashboard.tsx` to include tab selector
- [ ] Persist selected IDE tab
- [ ] Handle switch confirmation with IDE running check

## Success Criteria

- IDE tabs visible in dashboard
- Only installed IDEs shown
- Switching tabs shows correct profile list
- Can save/switch/delete IDE profiles from UI
- Claude Code tab works exactly as before
