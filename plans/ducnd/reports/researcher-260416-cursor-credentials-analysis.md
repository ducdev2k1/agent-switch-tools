# Cursor IDE Authentication Credentials Storage: Technical Analysis

**Date:** 2026-04-16  
**Research Focus:** Exact file paths, database schemas, authentication mechanisms for Cursor IDE credential storage  
**Target Use Case:** Building a credential management tool similar to `claude-tools` but for Cursor IDE

---

## 1. AUTHENTICATION ARCHITECTURE OVERVIEW

### High-Level Flow
```
User Login (Browser) 
  ↓
OAuth 2.0 Flow (Cursor Auth Server)
  ↓
Token Generation (accessToken + refreshToken)
  ↓
Storage in state.vscdb (SQLite Database)
  ↓
Persistent Session + Auto-Refresh
```

**Key Insight:** Cursor does NOT use a `.credentials.json` file like Claude Code. It exclusively uses SQLite state.vscdb.

---

## 2. EXACT FILE PATHS BY OPERATING SYSTEM

### macOS
```
~/.config/Cursor/User/globalStorage/state.vscdb         ❌ WRONG
~/Library/Application Support/Cursor/User/globalStorage/state.vscdb  ✓ CORRECT
```

**Full Expanded Path:**
```
/Users/{username}/Library/Application Support/Cursor/User/globalStorage/state.vscdb
```

**Backup File:**
```
/Users/{username}/Library/Application Support/Cursor/User/globalStorage/state.vscdb.backup
```

### Windows
```
%APPDATA%\Cursor\User\globalStorage\state.vscdb
```

**Full Expanded Path:**
```
C:\Users\{username}\AppData\Roaming\Cursor\User\globalStorage\state.vscdb
```

**Configuration File (Optional):**
```
%APPDATA%\Cursor\User\globalStorage\storage.json
```

### Linux
```
~/.config/Cursor/User/globalStorage/state.vscdb     ⚠️ THEORETICAL (may not work)
```

**Actual Behavior:** The search results do NOT confirm a standard Linux path. Linux has known keyring integration issues (see Section 5).

**Expected Path (inferred from macOS/Windows pattern):**
```
~/.local/share/Cursor/User/globalStorage/state.vscdb  (OR)
~/.config/Cursor/User/globalStorage/state.vscdb
```

---

## 3. DATABASE SCHEMA: state.vscdb

### Table Structure
```sql
CREATE TABLE ItemTable (
  key TEXT UNIQUE ON CONFLICT REPLACE,
  value BLOB
);
```

**Key Points:**
- Single table: `ItemTable`
- Column 1: `key` (TEXT, UNIQUE)
- Column 2: `value` (BLOB)
- Data stored as JSON serialized in BLOB field
- No encryption/obfuscation of plain data (credentials visible if blob decoded)

### Querying Example
```bash
sqlite3 "/Users/username/Library/Application Support/Cursor/User/globalStorage/state.vscdb" \
  "SELECT value FROM ItemTable WHERE key = 'history.recentlyOpenedPathsList'"
```

---

## 4. AUTHENTICATION CREDENTIAL KEYS IN state.vscdb

### Primary Authentication Keys (CRITICAL)

| Key Name | Data Type | Purpose | Notes |
|----------|-----------|---------|-------|
| `cursorAuth/accessToken` | String (JWT) | Active OAuth access token | Expires, used for API requests |
| `cursorAuth/refreshToken` | String (JWT) | OAuth refresh token | Used to obtain new accessToken |
| `cursorAuth/cachedEmail` | String | User's registered email | Cached for UI display |
| `cursorAuth/cachedSignUpType` | String | Auth method (e.g., "google", "github") | Sign-up provider identifier |
| `cursorAuth/stripeMembershipType` | String | Subscription tier | (e.g., "pro", "free", "enterprise") |

### Machine Identification Keys (IMPORTANT)

| Key Name | Data Type | Purpose | Notes |
|----------|-----------|---------|-------|
| `storage.serviceMachineId` | String (UUID) | Machine fingerprint | Used for token/trial management |
| `telemetry.devDeviceId` | String (UUID) | Development device ID | Telemetry tracking |
| `telemetry.macMachineId` | String | macOS-specific machine ID | macOS only |
| `telemetry.machineId` | String (UUID) | Generic machine ID | Cross-platform |
| `telemetry.sqmId` | String (UUID) | SQM (Software Quality Metrics) ID | Analytics |

### Chat History Keys (SECONDARY)

| Key Name | Data Type | Purpose | Notes |
|----------|-----------|---------|-------|
| `composer.composerData` | JSON | Primary chat list (new format) | Active conversations |
| `workbench.panel.aichat.view.aichat.chatdata` | JSON | Legacy chat storage | Deprecated format |
| `aiService.prompts` | JSON Array | Prompt history | User-created prompts |
| `aiService.generations` | JSON Array | Response history | Model-generated content |

---

## 5. TOKEN REFRESH MECHANISM

### OAuth 2.0 Refresh Flow
**Endpoint:** `https://api2.cursor.sh/oauth/token` (inferred from search results)

**Request Method:** POST

**Payload Structure:**
```json
{
  "grant_type": "refresh_token",
  "client_id": "{cursor_client_id}",
  "refresh_token": "{stored_refreshToken}",
  "code_verifier": "{pkce_verifier}"
}
```

**Response Structure:**
```json
{
  "access_token": "new_jwt_token",
  "refresh_token": "new_refresh_token",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

**Auto-Refresh Behavior:**
- Cursor automatically refreshes tokens **before expiration**
- No manual intervention needed
- Refresh tokens stored persistently in state.vscdb
- If refresh fails, user is logged out and forced to re-authenticate

---

## 6. CREDENTIAL EXTRACTION & MANIPULATION

### Reading Credentials Programmatically

**SQLite Query:**
```bash
sqlite3 "/Users/username/Library/Application Support/Cursor/User/globalStorage/state.vscdb" \
  "SELECT key, value FROM ItemTable WHERE key LIKE 'cursorAuth/%';"
```

**Python Example:**
```python
import sqlite3
import json

db_path = "/Users/username/Library/Application Support/Cursor/User/globalStorage/state.vscdb"
conn = sqlite3.connect(db_path)
cursor = conn.cursor()

# Fetch authentication tokens
cursor.execute("SELECT key, value FROM ItemTable WHERE key LIKE 'cursorAuth/%'")
for key, value in cursor.fetchall():
    print(f"{key}: {value.decode('utf-8') if isinstance(value, bytes) else value}")

conn.close()
```

### Modifying Credentials

**WARNING:** Direct database modification is risky:
1. Cursor must be COMPLETELY CLOSED before modifications
2. Always create `state.vscdb.backup` first
3. Use transactions to avoid corruption

**Update Query:**
```sql
UPDATE ItemTable SET value = '...' WHERE key = 'cursorAuth/accessToken';
```

### Backup & Restore Strategy
```bash
# Backup
cp ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
   ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb.backup

# Restore
cp ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb.backup \
   ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb

# Verify backup
sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb.backup \
  "SELECT key FROM ItemTable WHERE key LIKE 'cursorAuth/%';"
```

---

## 7. MULTI-ACCOUNT SWITCHING SUPPORT

### Current State
- **Cursor officially does NOT support multi-account switching**
- Users must log out/log in to switch accounts
- This is a frequently requested feature (multiple GitHub issues)

### Workaround Techniques

#### Option A: Isolated User Data Directories
```bash
# Run separate Cursor instances with different accounts
cursor --user-data-dir=/path/to/account1/data
cursor --user-data-dir=/path/to/account2/data
```

**Limitations:**
- Requires running multiple Cursor instances
- Each instance has separate settings/chat history
- No built-in account switcher

#### Option B: State Database Switching
Create separate database snapshots per account:

```bash
# Account 1 - Backup
mv ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
   ~/cursor-accounts/account1-state.vscdb

# Account 2 - Activate
cp ~/cursor-accounts/account2-state.vscdb \
   ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb

# Restart Cursor
```

**Limitations:**
- Chat history switches with account (no unified view)
- Manual process (prone to errors)
- Cursor caches data in memory (must restart)

#### Option C: Third-Party Tools
Three community tools currently exist:

1. **cursor-token-manager** (GitHub: LukaPetrovic24)
   - Desktop app for macOS
   - Features: Switch accounts, reset machine code, clean chat records
   - Implementation: Direct state.vscdb manipulation

2. **cursor-auto-account** (GitHub: Salimsalim1997)
   - Web interface + backend service
   - Features: Auto-registration, account status management, encrypted storage
   - Implementation: Manages multiple database snapshots

3. **cursor-reset** (GitHub: ultrasev)
   - Python script (all platforms)
   - Features: Reset device ID, bypass trial limits, switch accounts
   - Manipulates keys: `telemetry.*`, `storage.serviceMachineId`

---

## 8. LINUX-SPECIFIC AUTHENTICATION ISSUES

### Critical Problem
- **Cursor fails to identify OS keyring** (GNOME Keyring, KDE Wallet, etc.)
- Authentication URI received by Cursor browser
- Token storage **blocked by keyring identification error**
- Secure credential storage fails silently

### Current Workarounds
```bash
# Ensure GNOME Keyring is unlocked
export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u)/bus

# Export GPG/SSH agent socket
export GPG_AGENT_INFO="/run/user/$(id -u)/gnupg/S.gpg-agent:0:1"

# Start Cursor with keyring available
gnome-keyring-daemon --start --foreground &
cursor
```

### Implications for Credential Management
- Cannot reliably use OS keyring storage on Linux
- May fall back to plain-text storage in state.vscdb
- Database file becomes single point of failure for credentials

---

## 9. COMPARISON: Cursor vs. Claude Code Authentication

| Aspect | Cursor IDE | Claude Code |
|--------|-----------|-------------|
| Credential Storage | SQLite (state.vscdb) | `.credentials.json` file + OS keyring |
| Database vs. File | Single SQLite DB | Plain JSON + encrypted keyring |
| Multi-Account Support | No (official) | Yes (built-in) |
| Account Switching | Manual login/logout | CLI: `claude switch` or UI menu |
| Token Persistence | OAuth tokens in DB | API keys + session tokens in keyring |
| Refresh Mechanism | Auto-refresh (OAuth 2.0) | Session token with TTL |
| Backup Strategy | state.vscdb.backup file | `~/.claude/` directory |
| Encryption | No (plain BLOB) | OS keyring encryption |
| Third-Party Tools | cursor-token-manager, cursor-reset | claude-tools (this project) |

---

## 10. CRITICAL TECHNICAL INSIGHTS

### Security Considerations
1. **No database encryption:** Credentials visible in BLOB if extracted
2. **Machine ID dependency:** `storage.serviceMachineId` tied to device fingerprint (reset = new account)
3. **Token expiration:** Short-lived accessToken (auto-refreshed), long-lived refreshToken (risky if leaked)
4. **Keyring on Linux:** Unreliable; falls back to plain-text storage

### Backward Compatibility
- state.vscdb format stable across versions (at least v2.6.22 ↔ current)
- ItemTable schema unchanged (likely won't change)
- Keys may be added (e.g., new subscription fields) but old keys preserved

### Account Switching Challenges
1. **Machine ID tied to account:** Switching requires generating new `storage.serviceMachineId`
2. **Chat history coupling:** All chats stored in same database
3. **Trial system:** Machine ID reset detected by Cursor backend = new trial eligibility
4. **No official API:** No public method to enumerate/switch accounts

---

## 11. UNRESOLVED QUESTIONS

1. **Linux path confirmation:** Is state.vscdb actually stored in `~/.config/Cursor/...` or `~/.local/share/Cursor/...`? Search results unclear.
2. **Keyring integration:** Exact error handling when keyring fails on Linux — does Cursor fall back to plain-text?
3. **storage.json purpose:** Windows `storage.json` file — what's the relationship to state.vscdb? Redundant or different purpose?
4. **Token encryption in BLOB:** Are token values stored as plain UTF-8 strings or Base64-encoded? Testing needed.
5. **API endpoint URLs:** Exact hostname of Cursor OAuth server — is it `api2.cursor.sh` or just `cursor.sh`? (Inferred from search, not confirmed)
6. **Client ID/Secret:** Where is Cursor's OAuth `client_id` stored? In-app hardcoded or in database?
7. **Backward account switching:** If switching from Account A → B → A, is the original refreshToken still valid, or does Cursor invalidate old tokens?

---

## 12. RECOMMENDATIONS FOR CLAUDE-TOOLS CURSOR EXTENSION

### Approach 1: File-Based Account Switching (RECOMMENDED)
**Pro:**
- Simple, no risk of database corruption
- Works across all platforms
- Easy backup/restore

**Con:**
- Manual restart required
- Chat history doesn't persist across switches

**Implementation:**
```python
def switch_cursor_account(account_name):
    backup_path = f"~/.cursor-accounts/{account_name}/state.vscdb"
    active_path = "~/Library/Application Support/Cursor/User/globalStorage/state.vscdb"
    
    # Ensure Cursor is closed
    os.system("killall Cursor")
    
    # Swap database
    shutil.copy(backup_path, active_path)
    
    # Restart Cursor
    subprocess.Popen(["/Applications/Cursor.app/Contents/MacOS/Cursor"])
```

### Approach 2: Machine ID Reset (RISKY)
**Pro:**
- Allows "fresh account" on same machine without database swap
- Useful for bypassing trial limits (controversial)

**Con:**
- Detectable by Cursor backend
- May violate terms of service
- Unreliable on Linux

**Implementation:** See cursor-reset projects for reference

### Approach 3: Hybrid (IDEAL)
1. **Primary:** File-based switching (Approach 1)
2. **Secondary:** Optional machine ID management (Approach 2)
3. **Metadata:** Store account labels in separate JSON (outside state.vscdb)

---

## SUMMARY TABLE: Exact File Paths & Keys

| OS | Database Path | Backup Path | Key Prefix | Status |
|---|---|---|---|---|
| macOS | `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` | `...state.vscdb.backup` | `cursorAuth/*` | Confirmed |
| Windows | `%APPDATA%\Cursor\User\globalStorage\state.vscdb` | `...state.vscdb.backup` | `cursorAuth/*` | Confirmed |
| Linux | Unknown (likely `~/.local/share/Cursor/...`) | Unknown | `cursorAuth/*` | Unconfirmed |

---

## Sources

- [Authentication Management - Ryan0204/cursor-auto-icloud](https://deepwiki.com/Ryan0204/cursor-auto-icloud/5.2-authentication-management)
- [How To Restore Cursor User Rules](https://meirg.co.il/2025/05/31/how-to-restore-cursor-user-rules/)
- [Transferring Cursor Authentication Session Between Computers](https://medium.com/@botsmanp/transferring-a-cursor-authentication-session-between-computers-29ecb1ec2d1f)
- [Trying to change Cursor settings without clicking?](https://www.jackyoustra.com/blog/cursor-settings-location)
- [Support Multiple Authenticated Accounts - Cursor Forum](https://forum.cursor.com/t/support-multiple-authenticated-accounts-quick-account-switching/149020)
- [cursor-token-manager GitHub](https://github.com/LukaPetrovic24/cursor-token-manager)
- [cursor-reset GitHub](https://github.com/ultrasev/cursor-reset)
- [Cursor Account Switcher VS Code Extension](https://marketplace.visualstudio.com/items?itemName=AliAldahmani.cursor-account-switcher)
- [How to manage multiple Cursor accounts](https://appliscale.io/managing-multiple-cursor-accounts-a-practical-solution-for-agencies/)
- [Cursor IDE Security Best Practices](https://www.backslash.security/blog/cursor-ide-security-best-practices)
- [Configuration Cursor Docs](https://cursor.com/docs/cli/reference/configuration)
- [Best practices with state.vscdb](https://forum.cursor.com/t/best-practices-with-state-vscdb-and-state-vscdb-backup-in-cursor/156848)
- [MCP Authentication in Cursor 2026 Guide](https://www.truefoundry.com/blog/mcp-authentication-in-cursor-oauth-api-keys-and-secure-configuration)
- [VS Code Extension Storage Explained](https://medium.com/@krithikanithyanandam/vs-code-extension-storage-explained-the-what-where-and-how-3a0846a632ea)
- [Exploring VS Code Global State](https://mattreduce.com/posts/vscode-global-state/)
