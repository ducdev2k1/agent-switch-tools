# Claude Tools v1.0.8 Release Notes

**Claude Tools v1.0.8** adds per-device usage tracking, session usage webhook reports, and automatic quota refresh for all profiles.

## What's New?

### 1. Device Identity

- **Unique UUID**: Each machine generates a `device_id` (UUID v4) on first launch, stored at `~/.claude/.claude-tools/device.json`. This ID never changes.
- **Rename Your Device**: Go to Settings > Device to view Device ID, hostname, and rename the device (defaults to hostname).
- **Attached to Webhooks**: Device info is automatically included in session usage webhook payloads, making it easy to identify which machine sent the data.

### 2. Session Usage Tracking

- **JSONL Session Parsing**: The Rust backend scans all `.jsonl` files under `~/.claude/projects/`, aggregating input/output tokens, cache read/write, and message counts per session.
- **Preview Before Sending**: Click "Preview" in Settings to see a list of sessions with token usage before sending.
- **Webhook Reports**: Choose a time period (1h, 5h, 24h, 7d) and detail level (summary or detailed), then send a report to your configured webhook URL. Payload includes: device info, member email, aggregate totals, and per-session breakdowns.

### 3. Background Auto-Refresh for All Profiles

- **Background Worker**: Every 5 minutes, the app automatically calls the Anthropic Usage API to refresh quota for all profiles (active + saved) — no manual refresh needed.
- **Event-Driven UI**: The frontend listens for `all-profiles-usage-updated` and `usage-updated` events to update in real time, eliminating polling.
- **Rate-Limit Safe**: 1-second delay between API calls to avoid hitting rate limits.

### 4. CI/CD Improvements

- **Parallel Builds**: The release workflow now creates a draft release in a separate initial step, allowing platform builds (Ubuntu, macOS Intel, macOS ARM, Windows) to run concurrently. This reduces total release time.

---

_v1.0.8 lets you track token usage per device and session, while quota stays up to date automatically without manual intervention._
