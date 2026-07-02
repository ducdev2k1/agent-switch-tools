# Agent Switch Tools v1.0.15 Release Notes

**Agent Switch Tools v1.0.15** fixes a critical bug where logins performed outside the app went undetected (and could destroy the previous account's backup), keeps account backups always fresh, turns the activity log into a proper table, and removes the UI flash on window focus.

## What's New?

### 1. External logins are detected again (critical fix)

Since v1.0.11 the app read the account identity from its own cache in `~/.claude.json` instead of the `oauthAccount` field written by Claude Code. As a result, when you ran `claude /login` with a different account:

- The app **didn't notice** the switch — the old account still showed as ACTIVE.
- The new account **never appeared** in the list.
- Worst case: pressing **"Save Current"** at that point **overwrote the old account's backup with the new account's tokens**.

This release reads the source of truth written by Claude Code:

- External logins are detected on the next app open/focus: the new account is auto-saved as a profile (with full identity), the previous one stays intact, and a notification appears.
- Switching inside the app now always rewrites the target account's identity — eliminating false drift detection that could overwrite the wrong folder.
- Account cards show the **display name** and **organization name** again (lost since v1.0.11).

> **Note:** if you pressed "Save Current" after an external login before updating, that account's backup may hold the wrong tokens. Run `claude /login` for that account once — the app will re-save it correctly.

### 2. Backups are always the freshest available

Claude Code rotates tokens while you work, so a backup taken at the last switch could be stale. Now the active account's backup and identity are synced with the live state **on every app open/focus** — when you log into another account, the preserved snapshot of the old one is as fresh as possible.

### 3. Activity log as a table

The **Auto Session** tab's activity log moved from raw text to a **4-column table** (Time / Account / Status / Detail):

- Newest entries first, sticky header while scrolling.
- Timestamps formatted **dd/mm/yyyy hh:mm** — including reset times inside details, converted to your local timezone (no more `2026-06-30T11:09:59+00:00`).
- Status rendered as colored badges (success / hold / failed / skip).

### 4. No more UI flash on window focus

- Previously, re-focusing the window swapped the account list for skeletons for a beat before re-rendering — the account info and quota "blinked". Data now stays on screen and refreshes silently in the background.
- Quota auto-refreshes on focus, **throttled to once per 2 minutes** (matching the backend's 2-minute cache) — switching between windows within 2 minutes sends zero requests to the Anthropic API.

---

## Notes

- Account detection/saving runs when you open or focus the app window (not in the background tray) — after `claude /login` with another account, open the app once so it registers the change.
- If the app never runs between two consecutive external logins, the account in between cannot be saved (the app never saw its tokens).

_v1.0.15 focuses on preserving account data across external logins and smoothing out the UI._
