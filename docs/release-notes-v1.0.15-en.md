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

### 5. Usage statistics are accurate again

The **Usage** tab had two bugs skewing its numbers:

- **Tokens were double-counted (~2x)**: streaming rewrites the same message on several log lines, each carrying the identical usage object — the old parser summed them all (measured ~53% duplicate usage lines on real data). Each message is now counted exactly once.
- **Wrong model attribution**: a whole session was labeled with the first model seen in the log, while sessions can span several models (main model, Haiku subagents, mid-session `/model` switches); sessions starting with a `<synthetic>` placeholder even lost their cost entirely. Tokens and costs are now **broken down per actual model**, and sessions are labeled by the model with the most tokens.

The statistics are also **closer to real time** now:

- **The "Today" card counts every log line since local midnight** instead of only sessions *started* today — sessions spanning midnight are no longer dropped (measured: the old method missed up to 75% of the day's tokens).
- The Usage tab **silently refreshes when the window regains focus** (no dimming, local log parsing only — no API calls), on top of the background worker's 5-minute cadence.

After updating, the Usage tab totals will **drop noticeably** (no more double counting) while the "Today" card may **go up** (no more dropped sessions) — those are the correct numbers.

---

## Notes

- Account detection/saving runs when you open or focus the app window (not in the background tray) — after `claude /login` with another account, open the app once so it registers the change.
- If the app never runs between two consecutive external logins, the account in between cannot be saved (the app never saw its tokens).

_v1.0.15 focuses on preserving account data across external logins and smoothing out the UI._
