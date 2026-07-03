# Agent Switch Tools v1.0.17 Release Notes

**Agent Switch Tools v1.0.17** fixes a fundamental issue on **macOS**: since Claude Code 2.x, Claude Code credentials live in the **macOS Keychain**, not in the file `~/.claude/.credentials.json`. Earlier versions only read/wrote the file, so on macOS account switching didn't actually take effect. This release routes the active account's entire credential flow through the Keychain on macOS, while Linux/Windows stay exactly as before.

## What's New?

### 1. macOS Keychain support for the active account

On macOS, the app now reads and writes the **active** account's credentials directly in the login Keychain — the exact place the Claude Code CLI reads from:

- Uses Claude Code 2.x's keychain naming: `Claude Code-credentials-<config-dir hash>`, with a fallback to the legacy name and to the file.
- Safe writes: updates in place, **reads back to confirm** the write landed on the right slot, and **retries up to 3 times** if the Keychain is momentarily locked.
- **Keychain ↔ file mirror**: if a credentials file already exists, the app also writes a copy to the file so background tasks can still read it while the Keychain is locked — but it never creates a new plaintext file where none existed.

### 2. Every action targets the right account on macOS

After this release, the following features on macOS read/write the correct target instead of missing a non-existent file:

- Showing and identifying the **active** account in the list and tray.
- **Saving** the current account as a profile.
- **Switching** accounts — the new credentials are actually used by Claude Code after a restart.
- **Reconciling** state when you log in via `claude /login` outside the app.
- **Token refresh**, **quota** reads, **priming**, and data sent via **webhook**.

### 3. Linux & Windows: unchanged

On Linux and Windows the app keeps using the file `~/.claude/.credentials.json` exactly as before — no behavior change. Saved profiles (under `~/.agent-switch-tools/`) remain plain files on every OS.

---

## Notes

- On macOS, the first Keychain write may prompt the OS for Keychain access — this is expected.
- If you previously ran an older build on macOS and the active account didn't show up, the app will now read it from the Keychain after updating.
- After switching, if Claude Code is running, restart it to use the new account.

_v1.0.17 focuses on making Claude Code account switching work correctly on macOS by storing credentials where Claude Code actually reads them — the login Keychain._
