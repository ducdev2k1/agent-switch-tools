# Agent Switch Tools v1.0.12 Release Notes

**Agent Switch Tools v1.0.12** brings full multi-variant support for Antigravity — Desktop, IDE, and CLI are now managed independently — and updates the quota engine to Antigravity's new **Weekly + 5-hour** rate-limit model. Tab switching is now instant (no reload flash), and OAuth credentials load automatically at build time.

## What's New?

### 1. Antigravity: Desktop, IDE & CLI — All Supported

Google split Antigravity into three separate products, each storing credentials differently. The app now detects and manages all three:

| Variant | Source |
|---------|--------|
| **Antigravity** (Desktop) | `~/.config/Antigravity` → `state.vscdb` |
| **Antigravity IDE** | `~/.config/Antigravity IDE` → `state.vscdb` (OAuth proto, no `antigravityAuthStatus`) |
| **Antigravity CLI** | `~/.gemini/antigravity-cli/antigravity-oauth-token` (JSON file) |

A new `CredentialSource` abstraction lets the same profile/switch/quota flow work across both VS Code `state.vscdb` stores and the CLI's plain JSON token file.

### 2. Grouped Tab with Variant Sub-Tabs

The three Antigravity variants now live under a **single "Antigravity" tab** in the top bar, with **Desktop / IDE / CLI sub-tabs** inside — instead of cluttering the bar with three near-identical icons.

### 3. New Quota Model — Weekly + 5-Hour Limits

Following Google's Gemini plan update, Antigravity now meters usage with **per-group Weekly and 5-hour rate limits**. The dashboard now reads the current `retrieveUserQuotaSummary` endpoint (the same one the native `usage` command uses) and shows:

- **Gemini — Weekly Limit / Five Hour Limit**
- **Claude and GPT — Weekly Limit / Five Hour Limit**

This replaces the older single-window per-model `remainingFraction` and applies to all three variants.

### 4. Account Identity for IDE & CLI

The newer Antigravity builds no longer store the account email locally. The app resolves it via Google's `userinfo` endpoint (using the account's own OAuth token) and caches it into the saved profile — so your email and avatar show correctly across Desktop, IDE, and CLI.

### 5. Instant Tab Switching — No Reload, No Layout Jump

Switching tabs no longer refetches from scratch (which caused a skeleton flash and layout shift). Profiles and quota are now cached in-memory and shown immediately, refreshing quietly in the background.

### 6. Robust OAuth Token Parsing

The Antigravity OAuth-token protobuf parser now locates the token by its sentinel key instead of by position, so it keeps working even when newer builds reorder the blob's fields.

### 7. Build-Time OAuth Credentials

OAuth client credentials are now loaded automatically from a local `.env` at build time (via `build.rs`), with CI continuing to inject them from secrets. No more empty-client refresh failures in local builds.

---

## Notes

- **Antigravity CLI quota requires an eligible account.** If Google reports *"Verify your account to continue"* (e.g. the account has no verified phone number), the quota API returns no data — for both the native CLI and this app — until the account is verified.
- Cursor and Windsurf are temporarily hidden from the dashboard.

_v1.0.12 makes every flavor of Antigravity — Desktop, IDE, and CLI — a first-class, observable account, aligned with Google's latest quota policy._
