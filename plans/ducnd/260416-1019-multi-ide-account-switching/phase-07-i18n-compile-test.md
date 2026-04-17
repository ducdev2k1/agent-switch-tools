# Phase 7: i18n + Compile + Test

**Priority:** Medium
**Status:** Pending
**Depends on:** Phase 5, Phase 6

---

## Overview

Them i18n keys cho IDE features, dam bao compile thanh cong, test toan bo flow tren dev machine.

## Related Code Files

**Modify:**
- `src/locales/en.json` — English translations
- `src/locales/vi.json` — Vietnamese translations

## i18n Keys Needed

```json
{
  "ide": {
    "tabs": {
      "claude_code": "Claude Code",
      "cursor": "Cursor",
      "antigravity": "Antigravity"
    },
    "labels": {
      "active_account": "Active Account",
      "saved_profiles": "Saved Profiles ({{count}})",
      "no_profiles": "No saved profiles",
      "no_profiles_info": "Save your current account to enable quick switching",
      "not_installed": "{{ide}} is not installed",
      "membership": "Membership"
    },
    "actions": {
      "save_current": "Save Current",
      "switch": "Switch",
      "rename": "Rename",
      "delete": "Delete"
    },
    "messages": {
      "close_ide_warning": "Please close {{ide}} before switching accounts",
      "switch_success": "Switched to '{{name}}' in {{ide}}",
      "save_success": "Saved current {{ide}} account: {{email}}",
      "delete_success": "Deleted profile '{{name}}' from {{ide}}",
      "ide_running_warning": "{{ide}} is running. Please close it and try again."
    },
    "errors": {
      "switch_failed": "Failed to switch: {{error}}",
      "save_failed": "Failed to save: {{error}}",
      "db_locked": "{{ide}} database is locked. Close {{ide}} and try again.",
      "not_logged_in": "No account is currently logged in to {{ide}}"
    }
  }
}
```

## Implementation Steps

1. Add all i18n keys to `en.json` and `vi.json`

2. Compile check:
   ```bash
   pnpm tauri:dev   # or pnpm build for faster check
   ```

3. Manual test on dev machine:
   - [ ] IDE detection works (Cursor + Antigravity shown)
   - [ ] Can view current Cursor account
   - [ ] Can save Cursor account as profile
   - [ ] Can switch Cursor profiles (with Cursor closed)
   - [ ] Can view current Antigravity account
   - [ ] Can save Antigravity account as profile
   - [ ] Can switch Antigravity profiles (with Antigravity closed)
   - [ ] Tray menu shows all IDEs
   - [ ] Quick-switch from tray works
   - [ ] Claude Code tab unchanged
   - [ ] i18n works (EN + VI)

## Todo

- [ ] Add i18n keys to en.json
- [ ] Add i18n keys to vi.json
- [ ] Compile Rust backend
- [ ] Compile frontend
- [ ] Manual test: IDE detection
- [ ] Manual test: profile CRUD for Cursor
- [ ] Manual test: profile CRUD for Antigravity
- [ ] Manual test: tray integration
- [ ] Manual test: Claude Code unchanged

## Success Criteria

- Project compiles without errors (Rust + TypeScript)
- All IDE features work end-to-end on dev machine
- i18n complete for EN + VI
- Claude Code functionality unchanged
- Tray menu shows all IDEs
