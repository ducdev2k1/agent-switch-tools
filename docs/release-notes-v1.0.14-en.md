# Agent Switch Tools v1.0.14 Release Notes

**Agent Switch Tools v1.0.14** fixes a critical bug that made token refresh always fail, improves the token-refresh button's feedback, and adds an **in-app changelog viewer** so you can see what changed in each update.

## What's new?

### 1. Fixed token refresh (HTTP 404)

Anthropic moved its OAuth endpoint, so the old path returned `404 Not Found`. As a result **every token refresh failed** — both the manual button and the automatic background refresh that runs during quota updates.

This release points to the live endpoint (`https://claude.ai/v1/oauth/token`):

- The manual token-refresh button works again.
- **Automatic token refresh is restored** — the app once again keeps the access token fresh when fetching usage, via the background worker (every 5 minutes), and during priming.

### 2. Clear feedback on token refresh

- When a refresh **fails**, the app now shows a **specific error message** (e.g. the token was revoked) instead of silently doing nothing.
- The token-refresh button (🔑) is now **always visible** on expired accounts — no need to hover over the card.

### 3. In-app changelog viewer

- Added a **"What's New"** entry under **Settings → About**: a quick, fully offline view of recent release highlights.
- When an update is available, the **update dialog** now shows the new version's release notes before you install.

---

## Notes

- If, after updating, a refresh still reports `400 invalid_grant` (not `404`), that account's refresh token has genuinely expired — please log in to that account again. The refresh mechanism itself is now correct.
- Refreshing a **saved (non-active) account** writes back to that account's own storage and never touches the active account.

_v1.0.14 focuses on restoring token refresh and helping you understand what each update brings._
