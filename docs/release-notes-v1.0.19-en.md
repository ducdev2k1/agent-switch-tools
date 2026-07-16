# Agent Switch Tools v1.0.19 Release Notes

**Agent Switch Tools v1.0.19** fixes account switching on **macOS** for profiles that were imported from another machine — the case where you zip a profile folder on one computer, copy it into the profiles directory on a Mac, and switching to it failed with **"Failed to write credential to macOS Keychain"** (sometimes with a Keychain password prompt on top). Self-logged-in profiles were unaffected, which made the bug especially confusing.

## What's New?

### 1. Fixed: switching to a shared/imported profile on macOS

Two independent problems stacked up on this path, and both are fixed:

- **A trailing newline broke write verification.** A `credentials.json` copied from another machine typically ends with a newline. The credential was written to the Keychain successfully, but macOS returns values containing non-printable characters hex-encoded on read-back, so the app's write verification never matched and the switch reported a failure. The credential is now stored trimmed, so the write/read round trip always matches.
- **Foreign-owned Keychain entries blocked silent updates.** If the Keychain entry was created by another app (for example the Claude Code CLI on first login), its access control list can reject an in-place update and trigger the macOS password prompt. The app now deletes and recreates the entry with its own access settings, so subsequent switches are silent.

### 2. Keychain operations can no longer hang the app

Previously, if macOS parked a Keychain call behind a security dialog, the app waited forever. Now:

- Every `security` call is capped at **15 seconds**.
- A blocked call returns a clear error naming the exact subcommand and likely cause (e.g. "timed out — likely blocked on a keychain dialog") instead of freezing the switch.
- Keychain error messages surface the real reason from macOS (locked keychain, denied access, timeout) instead of a generic failure line.

### 3. Real Keychain tests in CI

The exact failure scenarios above are now covered by integration tests that run against a real macOS Keychain on both Intel and Apple Silicon GitHub Actions runners, so account switching stays regression-free.

---

## Notes

- All changes are macOS-only; Linux and Windows keep using the credentials file exactly as before.
- If a switch still fails on your Mac, the error message now tells you the real cause — please include it when reporting.

_v1.0.19 makes switching to imported profiles work reliably on macOS and guarantees the app never freezes on a Keychain dialog._
