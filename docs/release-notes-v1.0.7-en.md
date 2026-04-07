# Claude Tools v1.0.7 Release Notes

**Claude Tools v1.0.7** introduces one-click OAuth token refresh and fixes stale cache issues when switching accounts.

## What's New?

### 1. OAuth Token Refresh

- **One-Click Refresh**: Expired profiles now show a "Refresh Token" button next to the expired badge. Click it to refresh the token — no re-login required.
- **Works for All Profiles**: Both the active account and saved (inactive) profiles can be refreshed directly from the dashboard.
- **CLI-Powered Refresh**: Uses `claude -p` under the hood to trigger the same refresh flow as the official CLI. For saved profiles, credentials are temporarily swapped, refreshed, then restored.

### 2. Account Switch Cache Fix

- **No More Stale Data**: Switching accounts previously showed cached usage data from the previous profile. The `useProfileUsage` hook now detects `isActive` changes, resets stale data, and force-refreshes from the API.
- **Force Refresh Restored**: Fixed a bug where the manual refresh button wasn't passing `forceRefresh` to the backend, causing it to always return cached results within the 2-minute TTL.

### 3. Backend (Rust)

- **New Module `token_refresh`**: Orchestrates token refresh via Claude CLI with credential swapping for saved profiles.
- **Two New Commands**: `refresh_active_token` and `refresh_profile_token` — callable from the frontend to refresh any profile's expired token.

### 4. Frontend Improvements

- **`useTokenRefresh` Hook**: New React hook wrapping both refresh commands with loading state.
- **`ProfileCard` Enhancement**: Inline "Refresh Token" action on expired profiles. After success, profiles list and usage data reload automatically.

---

_v1.0.7 removes the need to manually re-authenticate expired accounts. If a profile shows "Expired", one click brings it back to life._
