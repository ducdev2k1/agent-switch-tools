# IDE Credential Storage Research Report

**Date:** 2026-04-16  
**Scope:** Cursor, Windsurf/Codeium, Claude Code credential paths and authentication structures

---

## Executive Summary

Research identified exact filesystem paths and credential formats across three Claude-compatible IDEs. **Key finding:** Only Claude Code and Cursor use plaintext/accessible credential files. **Windsurf uses system keychain exclusively**, making credential access non-trivial for scripting.

---

## 1. Claude Code (Baseline Reference)

### Storage Locations
- **Config directory:** `~/.claude/` (override via `$CLAUDE_CONFIG_DIR`)
- **Credentials file:** `~/.claude/.credentials.json`
- **OAuth account file:** `~/.claude.json`
- **Additional files:** `~/.claude/settings.json`, `~/.claude/history.jsonl`

### Credential File Format (Linux/Windows)
```json
{
  "accessToken": "sk-ant-oat01-...",
  "refreshToken": "...",
  "expiresAt": "2027-02-18T07:00:00.000Z"
}
```

### Security
- **Linux:** File mode 0600 (user-only read/write)
- **Windows:** Inherits user profile ACLs
- **macOS:** Encrypted via system Keychain

### OAuth Token Identification
- Prefix `sk-ant-oat01-` indicates OAuth token
- Requires active Claude.ai Pro or Max subscription
- Environment variable override: `CLAUDE_CODE_OAUTH_TOKEN`

### Process Detection
```bash
pgrep -i "claude"
```

---

## 2. Cursor IDE

### Storage Locations

**macOS:**
```
~/Library/Application Support/Cursor/User/globalStorage/state.vscdb
~/Library/Application Support/Cursor/User/settings.json (user settings)
```

**Linux:**
```
~/.config/Cursor/User/globalStorage/state.vscdb
```

**Windows:**
```
%APPDATA%\Cursor\User\globalStorage\state.vscdb
```

### Credential Format
- **Type:** SQLite database (binary, not plaintext JSON)
- **File:** `state.vscdb`
- **Storage method:** Credentials stored as JSON blob in single SQLite table row
- **Accessibility:** Requires SQLite query tool to extract; human-unreadable as plaintext

### Project-Level Configuration
```
.cursor/rules/          ← Project-specific rules (.mdc format)
.cursorrules            ← Legacy configuration (deprecated)
```

### Rules Configuration Layers
1. Project rules: `.cursor/rules/*.mdc` (version-controlled per project)
2. User rules: Via GUI Settings > Rules
3. Legacy: `.cursorrules` (still supported, deprecated)

### Process Detection
```bash
pgrep -i "cursor"          # Generic match
pgrep "Cursor"             # Exact case match
# macOS: Also check for running Cursor.app
ps aux | grep -i "cursor"  # Full command line
```

### Key Limitation for Automation
**Cursor does NOT use plaintext credential files like Claude Code.** Extracting credentials programmatically requires:
- Reading SQLite database directly
- Parsing JSON blob from state.vscdb
- Handling binary format complexity

---

## 3. Windsurf (Codeium)

### Storage Locations

**macOS/Linux:**
```
~/.codeium/config.json              ← Primary config
~/.codeium/bin/{hash}/              ← Language server binaries
~/Library/Application Support/Windsurf/     ← Alternative macOS location
~/Library/Caches/Windsurf/                  ← macOS cache
```

**Windows:**
```
%LOCALAPPDATA%\Codeium\config.json
C:\Users\{username}\AppData\Local\Codeium\
```

### Credential Storage Strategy
**Windsurf intentionally does NOT store credentials in plaintext files.**

Instead:
- **System keychain (primary):**
  - macOS: Keychain Access
  - Linux: gnome-keyring / kwallet
  - Windows: Windows Credential Manager
- **config.json:** Contains configuration only, NOT credentials
- **Automatic discovery:** CSRF token, port, API key via gRPC from local language server (no OAuth flows)

### MCP Configuration
```
mcp_config.json    (NOT mcp.json like Cursor)
```

### Language Server Paths
```
~/.codeium/bin/39080e89780bea461f7a46e6dc1026d80a3a353a/language_server_linux_x64
~/.codeium/bin/{hash}/language_server_windows_x64.exe
~/.codeium/bin/{hash}/language_server_macos_{arch}
```

### API Key Discovery Method
When scripts need Windsurf credentials:
1. Local gRPC connection to Windsurf language server (no token needed)
2. Automatic credential propagation via CSRF token mechanism
3. NO direct file-based credential extraction possible

### Process Detection
```bash
pgrep -i "windsurf"
pgrep "codeium"
pgrep "language_server"  # Language server process
```

### Key Limitation for Automation
**Credentials stored in system keychain only.** To access programmatically:
- Linux: Use `secret-tool` (gnome-keyring) or `qdbus` (kwallet)
- macOS: Use `security` CLI tool
- Windows: Use Windows API or `cmdkey /list`

Example (Linux):
```bash
secret-tool lookup codeium api-key
```

---

## 4. Google Antigravity (Emerging Alternative)

### Storage Locations (Limited Documentation)

**macOS:**
```
~/Library/Application Support/Antigravity/User/keybindings.json
~/.gemini/antigravity/global_workflows/<NAME>.md
```

### Note
- Early/beta stage; minimal public documentation
- Not yet competitive with Windsurf feature-completeness
- Credential storage mechanism not yet clearly documented

---

## Comparison Matrix

| Attribute | Claude Code | Cursor | Windsurf |
|-----------|-------------|--------|----------|
| **Credentials file** | `~/.claude/.credentials.json` | `state.vscdb` (SQLite) | System keychain only |
| **File format** | JSON plaintext | Binary SQLite | Encrypted in OS keychain |
| **Plaintext accessible** | Yes (Linux/Windows) | No (database query needed) | No (requires `secret-tool`/`security` CLI) |
| **OAuth token prefix** | `sk-ant-oat01-` | Internal to SQLite blob | Stored in keychain, not visible |
| **MCP config file** | N/A | `mcp.json` | `mcp_config.json` |
| **Process name** | `claude` | `cursor` | `windsurf`, `codeium`, `language_server` |
| **Override env var** | `CLAUDE_CODE_OAUTH_TOKEN` | N/A | N/A |
| **Config dir override** | `CLAUDE_CONFIG_DIR` | N/A | N/A |

---

## Automation Implications

### For Credential Detection Scripts

**Most scriptable:** Claude Code
- Simple JSON read
- File permissions protect sensitive data
- Environment variable override available

**Moderately scriptable:** Cursor
- Requires SQLite parsing
- Accessible but not trivial
- No environment override

**Least scriptable:** Windsurf
- System keychain required
- Platform-specific CLI tools needed
- Intentional security design (no file-based fallback)

### Recommended Detection Strategy

```bash
# Detect all three IDEs running
pgrep -i "claude" && echo "Claude Code running"
pgrep -i "cursor" && echo "Cursor running"
pgrep -i "windsurf" && echo "Windsurf running"

# Read Claude Code credentials
cat ~/.claude/.credentials.json | jq '.accessToken'

# Read Cursor credentials (requires sqlite3)
sqlite3 ~/Library/Application\ Support/Cursor/User/globalStorage/state.vscdb \
  "SELECT * FROM ItemTable WHERE key='claude.auth'" | jq '.value'

# Read Windsurf credentials (requires platform-specific keychain CLI)
# Linux:
secret-tool lookup codeium api-key
# macOS:
security find-generic-password -s "windsurf" -w
```

---

## Security Posture Analysis

**Claude Code:** Plaintext on disk, encrypted only on macOS
- **Risk:** Credential exposure if system compromised
- **Mitigation:** File permissions (0600) on Linux/Windows

**Cursor:** Encrypted in SQLite database
- **Risk:** Lower; requires local SQLite access
- **Mitigation:** Database stored in user-protected directory

**Windsurf:** System keychain only (gold standard)
- **Risk:** Lowest; credentials never written to plaintext files
- **Mitigation:** OS-level encryption and access controls
- **Tradeoff:** Less convenient for automation scripts

---

## Findings & Recommendations

### Key Findings

1. **No shared credential format** — Each IDE uses different storage mechanism
   - Claude Code: JSON plaintext
   - Cursor: SQLite blob
   - Windsurf: System keychain

2. **Windsurf prioritizes security over convenience**
   - Credentials stored only in system keychain
   - Intentional design; not oversight
   - Requires platform-specific CLI for programmatic access

3. **OAuth token discovery** 
   - Claude Code: Prefix `sk-ant-oat01-` identifies OAuth tokens
   - Cursor/Windsurf: OAuth mechanism not exposed in plaintext

4. **Process names are unreliable for IDE detection**
   - Cursor: May run as background service without GUI
   - Windsurf: Language server process (`language_server`) may run independently
   - Claude Code: Dedicated `claude` process

### Recommendations for Credential Detection Tools

**If building IDE credential sync:**
1. Support Claude Code directly (JSON read)
2. Add Cursor support via SQLite query (complexity: moderate)
3. Support Windsurf via platform-specific keychain CLI (complexity: high)
4. Detect running IDEs via `pgrep` with fallback to process inspection

**If implementing secure credential storage:**
- Follow Windsurf's keychain-first design
- Avoid plaintext JSON fallback (only for backwards compatibility)

---

## Unresolved Questions

1. **Cursor SQLite schema:** What is the exact column name and table structure in `state.vscdb`? (Search result referenced "ItemTable" but exact credentials location not confirmed)

2. **Windsurf Windows Credential Manager:** Exact registry path or PowerShell command to query Windsurf credentials from Windows Credential Manager?

3. **Antigravity credential format:** What is the finalized credential storage mechanism for Google Antigravity IDE? (Not yet documented publicly)

4. **Claude Code Azure/Bedrock support:** How are non-Claude OAuth credentials stored in `~/.claude/.credentials.json`? (Research only confirmed Claude.ai credentials format)

---

## Sources

- [Claude Code Authentication Docs](https://code.claude.com/docs/en/authentication)
- [Cursor Configuration Reference](https://cursor.com/docs/cli/reference/configuration)
- [Cursor Security Best Practices](https://www.backslash.security/blog/cursor-ide-security-best-practices)
- [Windsurf Documentation](https://docs.windsurf.com/windsurf/getting-started)
- [Windsurf Auth Token Setup](https://windsurf.com/show-auth-token?state=xyz)
- [Claude API Auth 2026](https://lalatenduswain.medium.com/claude-api-authentication-in-2026-oauth-tokens-vs-api-keys-explained-12e8298bed3d)
- [OpenCode Claude Auth Integration](https://github.com/griffinmartin/opencode-claude-auth)
- [Windsurf CSRF Token Auth](https://github.com/rsvedant/opencode-windsurf-auth)
- [Complete Guide to MCP Config Files](https://mcpplaygroundonline.com/blog/complete-guide-mcp-config-files-claude-desktop-cursor-lovable)
- [pgrep Manual](https://man7.org/linux/man-pages/man1/pgrep.1.html)
- [Windsurf.exe Process Info](https://spyshelter.com/exe/codeium-exafunction-inc-windsurf-exe/)
