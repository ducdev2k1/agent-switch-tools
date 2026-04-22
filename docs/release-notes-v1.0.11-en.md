# Agent Switch Tools v1.0.11 Release Notes

**Agent Switch Tools v1.0.11** brings full quota visibility for Antigravity accounts (including saved/inactive profiles), richer reset-time display for Claude CLI, and a smarter cache fallback so the UI never blanks when the quota API is down.

## What's New?

### 1. Auto Token Refresh — Claude CLI & Antigravity

Both providers now auto-refresh expired OAuth tokens so saved profiles keep fetching quota indefinitely, not just the one currently active.

**Claude CLI** — when `accessToken` is within 5 minutes of expiry, the app POSTs the stored `refreshToken` to Anthropic's OAuth endpoint (`console.anthropic.com/v1/oauth/token`) and rewrites `credentials.json` atomically with the new token pair. Anthropic rotates refresh tokens per exchange, so the file must be updated every cycle.

**Antigravity** — same pattern via Google OAuth. Refresh credentials extracted from the IDE's protobuf blob at `antigravityUnifiedStateSync.oauthToken` with a built-in manual parser (no extra dependencies).

### 2. Antigravity Quota — Works for All Accounts, Not Just Active

- **OAuth refresh flow**: Saved Antigravity profiles can now fetch their own quota. When a profile's access token expires (Google OAuth tokens live ~1 hour), the app reads the refresh token from the stored protobuf blob and silently exchanges it for a fresh access token via Google's OAuth endpoint.
- **Per-model grouping**: Instead of a single blob of quota data, the dashboard now shows three buckets matching Antigravity's native IDE layout:
  - **Gemini Pro** (3.1 Pro High/Low + 3 Pro High/Low — shared rate-limit pool)
  - **Gemini Flash** (3 Flash + 3.1 Flash Lite + others)
  - **Claude / GPT** (Claude Sonnet/Opus 4.6 + GPT-OSS 120B — premium pool)
- **Inverted semantics**: Antigravity buckets show **remaining %** (100 = full quota, 0 = exhausted) with inverted bar colors (green → full, red → low). Claude CLI keeps its used-% convention.

### 3. Claude CLI — 12-Hour Clock for Reset Times

- Each usage row now shows both the relative countdown and the exact clock time — e.g. `R: 2h 15m (3:45 PM)`. Format follows your system locale (12-hour AM/PM).

### 4. Graceful Quota API Failures

- **Stale-cache fallback**: When the live quota API fails (network error, 401 after token revoke, 5xx), the dashboard keeps showing your last-known progress bars and percentages instead of going blank. No more empty cards after a transient network hiccup.
- Applies to both Claude CLI and Antigravity.

### 5. Unsupported Providers Surface Clearly

- Cursor and Windsurf profile cards now show **"Quota not available"** (italic muted text) instead of an empty space. These two IDEs don't expose a public single-user quota API today.

### 6. Backend Cleanup

- Module restructure: `src-tauri/src/ide/` → `src-tauri/src/modules/{core,providers,quota,shared}/` for clearer separation between IDE providers, shared infrastructure, and core commands.

---

_v1.0.11 closes the gap between the Antigravity native UI and Agent Switch Tools' dashboard — all your IDE accounts now stay observable regardless of which one is currently active._
