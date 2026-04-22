# Cursor & Windsurf Quota/Usage APIs Research

## Executive Summary

**Cursor**: No official public API for user quota/usage in current (v2.6+) versions. Enterprise teams use Admin API (`daily-usage-data` endpoint) but single-user quota visibility is limited to dashboard UI. Token extraction from `state.vscdb` is community-reverse-engineered; no official endpoint documented.

**Windsurf**: Enterprise-only API at `https://server.codeium.com/api/v1/` with `GetTeamCreditBalance` (quota balance) and `GetUsageConfig` (per-user caps). Requires service key in request body. No single-user personal quota endpoint documented.

Both APIs lack comprehensive single-user quota endpoints at feature parity with Anthropic (which provides `GET /api/oauth/usage`).

---

## Cursor IDE Quota API

### Current Status
- **No official single-user quota endpoint** confirmed in public docs
- Enterprise Admin API exists (`https://api.cursor.com/teams/daily-usage-data`) but requires team setup
- Dashboard shows usage at `https://www.cursor.com/api/usage` (fails without session)

### Known Endpoints (Reverse-Engineered / Enterprise Only)

| Endpoint | Method | Auth | Purpose | Status |
|----------|--------|------|---------|--------|
| `https://api2.cursor.sh/auth/full_stripe_profile` | GET | Bearer token | Subscription/payment status | Community-found, called every ~500ms by client |
| `https://api2.cursor.sh/oauth/token` | POST | Basic (refresh_token) | OAuth token refresh | Official |
| `https://api.cursor.com/teams/daily-usage-data` | POST | API key | Team daily usage metrics | Enterprise Admin API |
| `https://api.cursor.com/teams/usage-events` | GET/POST | API key | Granular team usage events | Enterprise Admin API |

**Note**: No documented single-user quota endpoint like Anthropic's `GET /api/oauth/usage`.

### Authentication

**For Enterprise Admin API:**
- Requires API key (generated in Cursor dashboard: Settings → Cursor Admin API Keys)
- Used as username in HTTP Basic Auth or passed as header (format unclear from docs)
- Only team administrators can create keys

**For Local User Token Access (State.vscdb):**
- Token stored in SQLite at: `state.vscdb` key `cursorAuth/accessToken`
- Related keys: `cursorAuth/refreshToken`, `cursorAuth/cachedEmail`, `cursorAuth/stripeMembershipType`
- Token format: JWT (requires conversion to session token per community tools)
- No official documentation for extraction or API usage

### Token Extraction (Community Implementation)

Tools like [cursor-credits](https://github.com/CaptainCodeAU/cursor-credits) and [cursor-usage-vscode-extension](https://github.com/YossiSaadi/cursor-usage-vscode-extension) extract tokens:
1. Read `state.vscdb` SQLite database
2. Extract JWT from `cursorAuth/accessToken` key
3. Convert JWT to session token (method not officially documented)
4. Make authenticated requests to usage API

**Gotcha**: Token extraction is community-reverse-engineered. No official SDK or documented API method exists.

### Response Schema (Inferred)

No official schema documented. From community discussions:
- `full_stripe_profile` likely returns subscription status + plan info
- Daily usage endpoint returns: `{ date, lines_added, lines_deleted, lines_suggested, ... }` (but has known bugs per forum posts)

**Missing**: Specific fields for fast request count, credit balance, reset times. This is a **critical gap**.

### Rate Limits & Caching

- `full_stripe_profile` is called aggressively (~every 500ms) by Cursor client → expect 429 limits
- No documented rate limit policy for user-facing quota API
- Recommendation: Cache responses for ≥60s to avoid throttling

---

## Windsurf IDE Quota API

### Current Status
- **Enterprise-only API** at `https://server.codeium.com/api/v1/`
- No single-user personal quota endpoint
- Quota tracked as token-based budgets (daily + weekly)
- Users see quota only via in-app meter or subscription page

### Known Endpoints (Enterprise API Only)

| Endpoint | Method | Auth | Purpose | Permissions |
|----------|--------|------|---------|-------------|
| `POST /GetTeamCreditBalance` | POST | Service key in body | Team credit balance + billing cycle | Billing Read |
| `POST /GetUsageConfig` | POST | Service key in body | Per-user add-on credit cap config | Billing Read |
| `POST /Analytics` | POST | Service key in body | Custom analytics queries (autocomplete, chat, PCW) | Analytics Read |
| `POST /GetUsageConfig` | POST | Service key in body | Usage config for team/group/user scope | Billing Read |

**No single-user quota endpoint** like Cursor's `auth/full_stripe_profile` or Anthropic's `/api/oauth/usage`.

### Authentication

**Service Key (Windsurf Enterprise API):**
- Generated via Windsurf admin dashboard (Enterprise plans only)
- Passed in **request body** (not headers), e.g.:
  ```json
  {
    "service_key": "sk_...",
    "team_id": "team_123",
    ...
  }
  ```
- Must have appropriate permissions (Billing Read, Analytics Read, etc.)

**For Local User Token Access (State.vscdb):**
- Windsurf stores credentials at keys:
  - `windsurfAuthStatus` (JSON with `userStatusProtoBinaryBase64`)
  - `codeium.windsurf-windsurf_auth` (JSON with `api_key`, `token`, or `accessToken`)
- Format: Opaque token string (not JWT)
- No official documentation for extraction

### Token Extraction (Not Documented Officially)

Community tools for token extraction don't appear to exist yet (unlike Cursor). Likely approaches:
1. Query VSCode storage at `~/.config/Code/User/globalStorage/Codeium...` or similar
2. Parse JSON from `windsurfAuthStatus` key
3. Use token directly with Windsurf API (format unknown)

**Gotcha**: No community tools reverse-engineered yet. Windsurf auth extraction is an **unsolved problem** for desktop app integration.

### Response Schema (Inferred)

**GetTeamCreditBalance** (from docs):
```json
{
  "prompt_credits_per_seat": number,
  "num_seats": number,
  "add_on_credits": number,
  "billing_cycle_start": timestamp,
  "billing_cycle_end": timestamp
}
```

**No schema documented** for:
- Single-user quota remaining
- Token-based budget breakdown
- Reset times per model or tier

### Rate Limits & Caching

- No documented rate limits for Windsurf API
- Enterprise API may have standard SaaS limits (100–1000 req/min typical)
- Recommendation: Cache responses for ≥300s (5 min) for team-level data

---

## Comparison: Anthropic vs Cursor vs Windsurf

| Aspect | Anthropic | Cursor | Windsurf |
|--------|-----------|--------|----------|
| **Single-user quota endpoint** | ✅ Official: `/api/oauth/usage` | ❌ Not public/documented | ❌ Enterprise only |
| **Auth method** | Bearer token (OAuth) | JWT from state.vscdb + session token | Service key in body |
| **Token storage** | Credentials file | SQLite state.vscdb | VSCode globalStorage (unknown) |
| **Response format** | ISO-8601 resets, 0–100 utilization | Unknown (not documented) | Only team-level (not user) |
| **Community tooling** | Mature (official) | Reverse-engineered (cursor-credits, etc.) | Minimal (no tools found) |
| **Rate limiting** | Documented | Not public | Not documented |
| **Maturity** | Production-ready | Community-maintained | Pre-release for public use |

---

## Unresolved Questions

1. **Cursor**: What is the exact format of the session token after JWT conversion? Is there an official API for conversion?
2. **Cursor**: Does `full_stripe_profile` return remaining fast requests? If not, where is that data sourced?
3. **Windsurf**: How are user tokens stored in VSCode globalStorage on each OS (macOS, Linux, Windows paths)?
4. **Windsurf**: Is there a personal/user-level quota API endpoint that's not in Enterprise docs?
5. **Both**: What are the exact rate limits? Both appear aggressive or undocumented.
6. **Both**: When are quotas reset (UTC? timezone-aware?)?
7. **Cursor**: Why is the Admin API documented but single-user API not? Is single-user quota intentionally not exposed?

---

## Recommendations for Integration

### For Cursor
1. **Do not rely on reverse-engineered token extraction** — too fragile. Instead:
   - Use Cursor's official APIs only (Auth, Admin if enterprise)
   - Or ask Cursor support for a public single-user quota endpoint (request feature)
   - Fall back to showing "Quota not available" in UI until official endpoint exists

2. If forced to use community tools:
   - Pin `cursor-credits` version, monitor for breaks after Cursor updates
   - Cache responses ≥120s to respect aggressive `full_stripe_profile` polling
   - Handle 401/403 gracefully (token may be expired or revoked)

### For Windsurf
1. **Enterprise API requires service key** — not suitable for consumer app
   - Cannot request user's service key without security risk
   - Recommendation: Mark Windsurf quota as "Not available (requires enterprise setup)"

2. If user has enterprise, provide UI field to:
   - Input service key securely (store in local keychain)
   - Select team scope for quota queries
   - Cache team quota ≥300s

3. **Do NOT attempt token extraction** — no community reference exists yet, risk of data corruption

### For Both
- Show quota as **"Last updated: X seconds ago"** with refresh button
- Gracefully degrade: Show "—" (unknown) instead of 0% if API unavailable
- Cache aggressively (≥120s) to respect rate limits
- Implement exponential backoff on 429/503 responses

---

## Sources

- [Cursor Pricing Explained 2026 — Vantage](https://www.vantage.sh/blog/cursor-pricing-explained)
- [Cursor Usage and Limits — Official Docs](https://cursor.com/help/models-and-usage/usage-limits)
- [Windsurf Quota-Based Usage — Official Docs](https://docs.windsurf.com/windsurf/accounts/quota)
- [Windsurf API Reference — Official Docs](https://docs.windsurf.com/plugins/accounts/api-reference/api-introduction)
- [Cursor Admin API — Official Docs](https://docs.cursor.com/en/account/teams/admin-api)
- [Peeking Under the Hood of Cursor's API Calls — Speedscale Blog](https://speedscale.com/blog/peeking-under-the-hood-of-cursor/)
- [cursor-credits — GitHub](https://github.com/CaptainCodeAU/cursor-credits)
- [cursor-usage (MCP server) — GitHub](https://github.com/ofershap/cursor-usage)
- [cursor_api_demo — GitHub](https://github.com/eisbaw/cursor_api_demo)
- [Cursor Community Forum (various threads on quota/usage)](https://forum.cursor.com)
