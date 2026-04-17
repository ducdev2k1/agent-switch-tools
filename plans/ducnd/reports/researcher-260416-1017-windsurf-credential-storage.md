# Windsurf/Antigravity Credential Storage: Technical Research Report

**Date:** 2026-04-16  
**Status:** Complete  
**Confidence:** High on platform-level mechanisms; Medium on exact file formats (implementation details not publicly documented)

---

## Executive Summary

Windsurf (rebranded from Codeium in April 2025, then forked by Google as Antigravity in November 2025) stores authentication credentials **platform-dependent**: using system keychains on production deployments, but **plaintext JSON files** in development/plugin configurations. **Critical security gap identified**: Vim/Neovim plugins store API keys in `~/.config/` (often published on GitHub), contradicting best practices.

Multi-account support requires **third-party tools** (WindsurfSwitch); no native built-in support.

---

## 1. Credential Storage by Platform

### 1.1 Linux

**Primary Storage (Main IDE):**
- **Location:** System keychain (GNOME Keyring or KWallet)
- **Service:** Uses Secret Service API / libsecret
- **Mechanism:** gnome-keyring-daemon or kwallet service must be running
- **File Location:** `~/.local/share/keyrings/` (GNOME Keyring persistent storage)
- **Issue:** On minimal setups or WSL2, if keyring daemon not running, token storage fails silently

**Configuration Directory:**
- **Path:** `~/.codeium/windsurf/` or `~/.codeium/`
- **Contents Known:**
  - `mcp_config.json` - MCP server configuration
  - `.codeiumignore` - ignore patterns
  - `memories/` - subdirectory for global rules

**Plaintext Fallback (Vim/Neovim):**
- **Path:** `~/.config/` (XDG Base Directory Specification)
- **File:** `config.json`
- **Format:** JSON with cached API key
- **Risk:** Frequently synced to GitHub in dotfiles repos → **API key leakage**

---

### 1.2 macOS

**Primary Storage:**
- **Location:** macOS Keychain (~/Library/Keychains/)
- **Service:** Keychain Access system service
- **Encryption:** Uses OS-derived encryption key from user login
- **Standard Path:** `~/Library/Keychains/` and `/Library/Keychains/`

**Configuration Directory:**
- **Path:** `~/.codeium/windsurf/` (same as Linux)
- **Access:** Via Keychain Access GUI or `security` CLI

**VS Code / Electron App Specific:**
- **Storage:** Electron's safeStorage API (newer builds)
- **Database:** SQLite database in user data directory
- **Encryption:** Keychain-derived encryption key + additional layer

---

### 1.3 Windows

**Primary Storage:**
- **Location:** Windows Credential Manager (Windows Credentials vault)
- **Path:** `%APPDATA%\Microsoft\Credentials\`
- **Service:** Windows Credential Manager service

**Configuration Directory:**
- **Path:** `%APPDATA%\Codeium\Windsurf\`
- **Equivalent:** `C:\Users\[YourUsername]\.codeium\windsurf\`
- **Contents:** Same as Linux/macOS (mcp_config.json, etc.)

**VS Code Extension:**
- **Storage:** Windows Credential Manager
- **Method:** Electron's safeStorage integration

---

## 2. File Formats & Structures

### 2.1 Credentials.json (When Stored as File)

**Known Location (from docs):** `~/.codeium/credentials.json`

**Format (Inferred from API responses):**
```json
{
  "api_key": "sk-ws-...",
  "name": "username"
}
```

**API Key Format:**
- **Prefix:** `sk-ws-` (Windsurf Codeium tokens)
- **Example:** `sk-ws-abcdef1234567890...`
- **Obtainable Via:** 
  - Browser: `https://codeium.com/show-auth-token` (returns Firebase token)
  - Paste into IDE command palette

**Authentication Flow:**
1. User visits https://codeium.com/show-auth-token
2. Browser performs OAuth → returns Firebase ID token
3. Client exchanges Firebase token for API key via POST to `https://api.codeium.com/register_user/`
4. Response: `{"api_key": "sk-ws-...", "name": "username"}`
5. Token stored in system keychain OR plaintext JSON

---

### 2.2 Config.json (Vim/Neovim Plugins)

**Location:** `~/.config/codeium/config.json` (or cache directory)

**Example Structure:**
```json
{
  "api_key": "sk-ws-...",
  "auth_token": "...",
  "config_path": "~/.cache/codeium/config.json"
}
```

**Security Note:** GitHub issue #16 (windsurf.vim) flagged this as risky—developers don't realize config dirs get synced to dotfiles repos.

---

### 2.3 MCP Config (mcp_config.json)

**Location:** `~/.codeium/windsurf/mcp_config.json`

**Format:**
```json
{
  "mcpServers": {
    "servername": {
      "disabled": false,
      "serverUrl": "http://localhost:3000",
      "headers": {},
      "authType": "Bearer"
    }
  }
}
```

---

## 3. Credential Storage by Interface

### 3.1 Main Windsurf IDE (Electron App)
- **Storage:** Platform keychain (macOS Keychain, Windows Credential Manager, Linux GNOME Keyring)
- **Encryption:** Double-encrypted (Electron safeStorage + OS keychain)
- **File Fallback:** Plaintext if keyring unavailable (silently fails on Linux if daemon not running)
- **Backup Method:** Manual export via "Show Auth Token" command → paste token into settings

### 3.2 VS Code / Cursor / VS Codium Extension
- **Storage:** VS Code's secret storage API
  - **macOS:** Keychain Access (encryption key stored in Keychain)
  - **Windows:** Credential Manager
  - **Linux:** Keyring via Secret Service API
- **Database:** `state.vscdb` (SQLite) stores encrypted secrets
- **Method:** Command Palette → "Codeium: Provide Authentication Token"

### 3.3 Vim/Neovim Plugin (windsurf.vim, windsurf.nvim)
- **Storage:** `~/.config/codeium/config.json` (plaintext JSON)
- **Manual:** `:Codeium Auth` command → paste token
- **Issue:** XDG config dirs commonly synced to GitHub → credential exposure risk

### 3.4 JetBrains IDEs
- **Storage:** JetBrains credential store (encrypted per IDE's own mechanisms)
- **Method:** Settings → AI Assistant → Paste token

---

## 4. Multi-Account Support

### 4.1 Native Support
**None.** Windsurf/Antigravity have **no built-in multi-account switching**.

### 4.2 Third-Party Solutions

#### WindsurfSwitch (GitHub: usernamedoxelghk/WindsurfSwitch)
**Most Popular Tool**

**How It Works:**
1. **UI:** VSCode/Windsurf extension panel
2. **Input:** Email + password (or saved accounts list)
3. **Action:** Auto-retrieves API key via Codeium auth API
4. **Storage:** Extension state (secure if using VS Code secret storage)
5. **Shortcut:** `Cmd+Alt+S` (Mac) or `Ctrl+Alt+S` (Windows) to cycle accounts
6. **Features:** Add, switch, delete accounts; persistent account list

**Installation:** VSIX extension or npm source

#### Windsurf Switcher Free (GitHub: 1837620622/Windsurf-Switcher-Free)
**Alternative (Chinese Documentation)**
- No heartbeat detection
- No auto-logout
- Complete localization
- Account list persistence

#### Cockpit Tools (GitHub: jlcodes99/cockpit-tools)
**Universal AI IDE Manager**
- Supports Windsurf + multiple IDEs
- Isolated user data directories per account
- Multi-instance management
- Quota monitoring

---

## 5. Antigravity (Google Rebrand, Nov 2025)

### 5.1 Relationship to Windsurf
- Google acquired Windsurf team for $2.4 billion (July 2025)
- Antigravity = heavily modified Windsurf fork (or Windsurf fork of VSCode)
- Delivered within 4 months of acquisition

### 5.2 Credential Storage Changes
**NOT DOCUMENTED.** Search results show security analysis but no official credential path migration.

**Inferred Locations (Based on Windsurf patterns):**
- Likely still uses platform keychains for main IDE
- Possible alternate config dirs: `~/.antigravity/`, `~/.gemini/`
- Workspace-specific: `.agents/` directories

**Known Difference:** Antigravity uses `~/.gemini/` for global rules/workflows (vs. `~/.codeium/` for Windsurf)

**Security Note:** Antigravity agents have terminal access without sandboxing → credential management more critical than Windsurf's human-in-the-loop model

---

## 6. Backup & Restore Workflow

### 6.1 Simple Token Export/Import

**Export Current Token:**
```bash
# Windsurf IDE: Settings → Show Auth Token
# Or CLI: https://windsurf.com/show-auth-token
# Browser returns: sk-ws-...
```

**Backup Process:**
```bash
# On old machine:
# Copy token from settings dialog

# On new machine:
# Command Palette → "Codeium: Provide Authentication Token"
# Paste token
```

### 6.2 Config Directory Backup

**Linux/macOS:**
```bash
# Backup entire config
cp -r ~/.codeium/windsurf ~/windsurf-config-backup

# Restore
cp -r ~/windsurf-config-backup ~/.codeium/windsurf
```

**Windows:**
```powershell
# Backup
Copy-Item -Path $env:APPDATA\Codeium\Windsurf -Destination $env:APPDATA\Windsurf-Backup -Recurse

# Restore
Copy-Item -Path $env:APPDATA\Windsurf-Backup -Destination $env:APPDATA\Codeium\Windsurf -Recurse
```

### 6.3 Keychain/Credential Manager Backup
**Not straightforward—OS-specific:**
- **macOS:** Export from Keychain Access GUI
- **Windows:** Backup Credential Manager (admin required)
- **Linux:** GNOME Keyring backup (gnome-keyring-daemon manages, no simple export)

---

## 7. Security Assessment

### 7.1 Best Practices (Implemented)
✅ Platform keychain usage for main IDE  
✅ Double encryption (OS + app layer)  
✅ No plaintext in database for Electron apps (when safeStorage used)

### 7.2 Known Gaps
❌ **Plugin Credential Storage:** Vim/Neovim store plaintext in `~/.config/` (often synced to GitHub)  
❌ **Silent Failures:** Linux keyrings fail silently if daemon not running  
❌ **No Account Isolation:** Single credential per Windsurf installation; third-party tools required for multi-account  
❌ **Antigravity Terminal Access:** No sandboxing on agent execution → higher credential exposure risk

### 7.3 GitHub Leakage Risk
- Developers share dotfiles publicly
- `~/.config/codeium/config.json` may contain `sk-ws-*` tokens
- Automated crawlers can detect and reuse leaked tokens

---

## 8. Code Examples for Access

### 8.1 Reading Linux Keychain (GNOME Keyring)

```python
# Using Python keyring library
import keyring

# Retrieve token
token = keyring.get_password("codeium", "api_key")

# List all entries
import subprocess
result = subprocess.run(["secret-tool", "search", "service", "codeium"], capture_output=True, text=True)
print(result.stdout)
```

### 8.2 Reading Windows Credential Manager

```powershell
# PowerShell: List all credentials
Get-StoredCredential | Where-Object { $_.UserName -like "*codeium*" -or $_.TargetName -like "*codeium*" }

# Get specific credential
cmdkey /list:codeium
```

### 8.3 Reading macOS Keychain

```bash
# List all entries
security dump-keychain -d login.keychain

# Search for codeium
security find-generic-password -s codeium

# Extract password
security find-generic-password -s codeium -w
```

---

## 9. Unresolved Questions

1. **Exact Antigravity Config Paths:** Official migration docs from Windsurf → Antigravity not found. Are credentials in `~/.antigravity/`, `~/.gemini/`, or still `~/.codeium/`?

2. **Electron SafeStorage Encryption Key:** Where is the encryption key stored when safeStorage is used? On Linux, is it still in gnome-keyring or plaintext?

3. **LSP Mode / Remote Credentials:** How are credentials handled when Windsurf connects to remote servers or LSP endpoints?

4. **Credential Rotation Policy:** Does Codeium/Windsurf rotate API keys periodically? How often?

5. **Full credentials.json Schema:** Only `api_key` and `name` documented. Are there additional fields (expiry, scopes, etc.)?

6. **Windows Safe Storage Alternative:** Does Windsurf use Electron's safeStorage on Windows, or always Credential Manager?

---

## 10. Recommendations for Credential Management

### For Users
1. **Use Platform Keychain:** Let Windsurf store in system keychain; don't copy tokens to dotfiles
2. **Multi-Account:** Use WindsurfSwitch extension for account switching (stores in VS Code secret storage)
3. **Backup Token:** Save `sk-ws-*` token in encrypted password manager, not plain files
4. **Check Linux:** Verify `gnome-keyring-daemon` running: `ps aux | grep keyring`

### For Developers Integrating Windsurf
1. **Never Deploy Token in Config:** Use env vars or system keychain, not plaintext JSON
2. **Respect XDG_CONFIG_HOME:** Follow Linux standards for credential storage locations
3. **Test Credential Isolation:** Each user/account should have isolated credentials
4. **Document Backup/Restore:** Provide clear instructions for credential migration

---

## Sources

- [Windsurf Docs - Getting Started](https://docs.windsurf.com/windsurf/getting-started)
- [Windsurf Docs - Advanced](https://docs.windsurf.com/windsurf/advanced)
- [Windsurf Issue #484 - Token Storage](https://github.com/Exafunction/windsurf.vim/issues/484)
- [Windsurf Issue #16 - API Key Security](https://github.com/Exafunction/windsurf.vim/issues/16)
- [VS Code Extension Storage Explained](https://medium.com/@krithikanithyanandam/vs-code-extension-storage-explained-the-what-where-and-how-3a0846a632ea)
- [WindsurfSwitch - Multi-Account Manager](https://github.com/usernamedoxelghk/WindsurfSwitch)
- [Windsurf Authentication Token - VS Code](https://windsurf.com/show-auth-token)
- [Codeium GNOME Keyring Fix](https://medium.com/@logins_39559/visual-studio-code-gnome-keyring-fix-for-codeium-and-probably-other-things-d3815217ef54)
- [Linux Keyring, GNOME Keyring, Secret Service, and D-Bus](https://rtfm.co.ua/en/what-is-linux-keyring-gnome-keyring-secret-service-and-d-bus/)
- [Electron Secure Storage](https://www.electronjs.org/docs/latest/api/safe-storage)
- [Node-keytar - Secure Credential Storage](https://github.com/atom/node-keytar)
- [Cursor vs Windsurf IDE Comparison](https://www.qodo.ai/blog/windsurf-vs-cursor/)
- [Google Antigravity vs Windsurf - Augment Code](https://www.augmentcode.com/tools/google-antigravity-vs-windsurf)
