# Claude Code CLI: Credentials Storage & Authentication on Linux

**Research Date:** 2026-04-01 | **Researcher:** duc-lta

---

## Executive Summary

Claude Code stores credentials in a single, well-defined file on Linux: `~/.claude/.credentials.json`. Credentials cannot be swapped casually—the active account is determined by the OAuth tokens stored in this single file. Token refresh is automatic. Account switching requires full re-authentication via browser login.

---

## 1. Credentials Storage Location

**Path:** `~/.claude/.credentials.json`

**Environment Override:** `$CLAUDE_CONFIG_DIR` (if set)

**File Permissions (Linux):** `0600` (read/write owner only, restricted access)

**Ownership:** Owned by the user running Claude Code

---

## 2. Credentials File Format (JSON Structure)

```json
{
  "claudeAiOauth": {
    "accessToken": "sk-ant-oat01-...",
    "refreshToken": "sk-ant-ort01-...",
    "expiresAt": 1775078964659,
    "scopes": [
      "user:file_upload",
      "user:inference",
      "user:mcp_servers",
      "user:profile",
      "user:sessions:claude_code"
    ],
    "subscriptionType": "team",
    "rateLimitTier": "default_claude_max_5x"
  },
  "organizationUuid": "4cbb44e2-b660-423a-a0ac-c6be3f92dd67"
}
```

### Key Fields Explained

| Field | Purpose |
|-------|---------|
| `accessToken` | OAuth bearer token (sk-ant-oat01-...). Valid for ~30 days by default. |
| `refreshToken` | Long-lived refresh token (sk-ant-ort01-...). Used to obtain new access tokens. |
| `expiresAt` | Unix timestamp (ms) when the access token expires. |
| `scopes` | Array of permission scopes granted to this token. |
| `subscriptionType` | Account tier: "team", "pro", "max", "enterprise", or "free". |
| `rateLimitTier` | Rate limit classification (e.g., "default_claude_max_5x"). |
| `organizationUuid` | Organization ID for team/enterprise accounts. |

**Note:** Only `claudeAiOauth` is present for Claude.ai OAuth logins. Other auth methods (Claude Console API, Azure, Bedrock, Vertex) use different credential structures.

---

## 3. Related Files & Configuration

| File | Purpose | Scope |
|------|---------|-------|
| `~/.claude/settings.json` | Global Claude Code user settings (not credentials) | User-wide config |
| `~/.claude/.ck.json` | Extended configuration (plan naming, locale, hooks) | User-wide config |
| `~/.claude/sessions/` | Session metadata (PID, sessionId, CWD, startedAt) | Per-session |
| `~/.claude/session-env/` | Session-specific environment snapshots | Per-session |
| `~/.claude/projects/` | Per-project settings & metadata | Per-project |
| `~/.claude/policy-limits.json` | Security policy restrictions (remote control, web setup) | User-wide |
| `~/.claude/.credentials.json` | **THE ONLY CREDENTIAL FILE** | User-wide, single account |

---

## 4. Active Account Determination

Claude Code determines the active account **solely by the credentials in `~/.claude/.credentials.json`**:

- **One file, one account per user:** The same Linux user cannot run multiple Claude Code sessions with different accounts simultaneously.
- **No account switching within a session:** Once Claude Code starts with loaded credentials, that account is active for the entire session.
- **Token validation:** Claude Code validates tokens on startup and refreshes them as needed.

---

## 5. Can You Simply Swap credentials.json?

**Short answer:** Technically yes, but not cleanly. **Recommended approach: use `/logout` and `/login`.**

### Why Direct File Swapping is Problematic

1. **Session cache isolation:** Each session stores environment snapshots in `~/.claude/session-env/` (keyed by sessionId). Swapping credentials.json doesn't invalidate these caches.

2. **Project-level metadata:** `~/.claude/projects/` stores per-project settings. Old project configs may reference the previous account's organizationUuid, causing metadata mismatches.

3. **File history leakage:** `~/.claude/file-history/` tracks edits per session. Mixing credentials without proper cleanup leaves historical traces.

4. **MCP server authentication state:** `~/.claude/mcp-needs-auth-cache.json` caches authentication requirements for MCP servers. This cache may not reflect the new account's permissions.

### Clean Account Switching Method

```bash
claude  # Start Claude Code
/logout  # Logs out current account, clears credentials
/login   # Opens browser for new account login
```

This approach:
- Clears credentials properly
- Invalidates session caches
- Ensures a fresh token set
- Handles organizationUuid updates automatically

---

## 6. Token Refresh Mechanism

### Access Token Refresh

**Automatic refresh:**
- Triggered when `accessToken` is within 5 minutes of `expiresAt`
- Also triggered on HTTP 401 (Unauthorized) response
- Uses `refreshToken` to obtain new `accessToken`
- No user interaction required

### Refresh Token Behavior

- **Lifetime:** Long-lived (typically months)
- **Rotation:** May rotate on each refresh (check token structure after refresh)
- **Invalidation:** Cleared only on `/logout` or browser-initiated logout

### Expiration Handling

If both access and refresh tokens expire:
- Claude Code prompts to re-authenticate via browser
- Cannot proceed without valid token
- Equivalent to `/logout` + `/login`

---

## 7. Authentication Precedence (Priority Order)

When Claude Code starts, credentials are selected in this order:

1. **Cloud provider env vars** (`CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `CLAUDE_CODE_USE_FOUNDRY`)
2. **`ANTHROPIC_AUTH_TOKEN` env var** (Bearer token, for gateway/proxy auth)
3. **`ANTHROPIC_API_KEY` env var** (Console API key, direct Anthropic API)
4. **`apiKeyHelper` script output** (Custom script for dynamic credentials)
5. **OAuth credentials from ~/.claude/.credentials.json** (Default for subscriptions)

**Important:** If `ANTHROPIC_API_KEY` is set, it overrides stored OAuth credentials (even if valid). This can cause authentication failures if the key is expired or belongs to a different organization.

---

## 8. File Protection & Security

**Linux-specific protections:**
- `.credentials.json` created with mode `0600` (owner read/write only)
- No automatic encryption on Linux (unlike macOS Keychain)
- Stored as plaintext JSON with unencrypted tokens
- **Risk:** Full token compromise if file is accessible

**Best practices:**
- Never share or version control `.credentials.json`
- Use restricted file permissions (verify `ls -la ~/.claude/.credentials.json` shows `600`)
- Rotate tokens via `/logout` + `/login` if compromise is suspected
- Use environment variables for API keys instead of stored OAuth in shared environments

---

## 9. Multi-Account Scenarios

### Scenario: Running Two Claude Code Sessions with Different Accounts

**Not possible natively.** Workarounds:

1. **Use different Linux users:** Each user has separate `~/.claude/` directory
   ```bash
   sudo -u user1 claude  # User 1's session
   # In another terminal:
   sudo -u user2 claude  # User 2's session
   ```

2. **Use custom config directory:**
   ```bash
   CLAUDE_CONFIG_DIR=/tmp/claude-account-2 claude  # Separate credentials
   ```

3. **Container/VM isolation:** Run separate Claude Code instances in isolated environments

---

## 10. Debugging Credential Issues

**Check current authentication status:**
```bash
claude
/status  # Shows active auth method and token validity
```

**Verify credentials file:**
```bash
cat ~/.claude/.credentials.json | jq .claudeAiOauth  # View OAuth details
ls -la ~/.claude/.credentials.json  # Check permissions
```

**Force re-authentication:**
```bash
rm ~/.claude/.credentials.json  # Dangerous: also removes tokens
claude  # Will prompt for login
```

**Check environment variables (may override credentials):**
```bash
echo $ANTHROPIC_API_KEY
echo $ANTHROPIC_AUTH_TOKEN
echo $CLAUDE_CODE_USE_BEDROCK
```

---

## Summary Table

| Question | Answer |
|----------|--------|
| **Where?** | `~/.claude/.credentials.json` or `$CLAUDE_CONFIG_DIR/.credentials.json` |
| **Format?** | JSON with `claudeAiOauth` object containing accessToken, refreshToken, expiresAt, scopes |
| **Single file?** | Yes, one file per Claude Code user |
| **Account switching?** | Use `/logout` → `/login`, not manual file swaps |
| **Token refresh?** | Automatic (every 5 min before expiry, or on 401 response) |
| **Multiple accounts simultaneously?** | No, use separate Linux users or `$CLAUDE_CONFIG_DIR` |
| **File permissions?** | `0600` (owner only) on Linux |
| **Encrypted?** | No (plaintext JSON on Linux; Keychain on macOS) |

---

## Unresolved Questions

- Does Claude Code rotate `refreshToken` values on each token refresh, or keep the same refresh token indefinitely?
- What is the exact default TTL for `accessToken` before expiration?
- Can custom credential scripts (`apiKeyHelper`) be used with OAuth credentials, or only with API keys?

---

## Sources

- [Claude Code Authentication Documentation](https://code.claude.com/docs/en/authentication)
- [Claude Code GitHub Issue: Credentials Storage Differences](https://github.com/anthropics/claude-code/issues/1414)
- Live inspection of `~/.claude/` directory structure on Linux (user: duc-lta)
- Actual `.credentials.json` file structure examined (Apr 1, 2026)
