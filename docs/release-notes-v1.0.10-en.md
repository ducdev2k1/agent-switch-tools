# Agent Switch Tools v1.0.10 Release Notes

**Agent Switch Tools v1.0.10** is a major rebrand and architectural overhaul. The app — previously known as **Claude Tools** — now supports switching accounts across multiple AI coding agents: **Claude Code**, **Cursor**, **Windsurf**, and **Antigravity**.

## What's New?

### 1. Rebrand: Claude Tools → Agent Switch Tools

- **New name**: Because the app no longer targets only Claude Code, we renamed it to reflect its true scope — managing multiple AI coding agent accounts from one place.
- **New identifier**: Bundle identifier changed to `com.ducdev2k1.agent-switch-tools`.
- **Repo moved**: New GitHub home at `github.com/ducdev2k1/agent-switch-tools`.
- **UI updated**: Window title, tray tooltip, settings page, and all surfaces now show "Agent Switch Tools".

### 2. Multi-IDE Account Switching

- **Cursor support**: Switch between multiple Cursor accounts by backing up and restoring auth keys from Cursor's `state.vscdb` (SQLite).
- **Windsurf support**: Full account management for Windsurf IDE with protobuf-based email extraction.
- **Antigravity support**: Manage multiple Antigravity accounts with JSON-field email extraction.
- **Auto-detection**: App automatically detects which IDEs are installed on your machine and shows them in the dashboard.
- **Per-IDE profiles**: Each IDE has its own isolated profile storage — no cross-contamination.

### 3. Unified Storage Structure

- **New root directory**: All app data now lives under `~/.agent-switch-tools/` (previously `~/.claude/.claude-tools/`).
- **Consistent layout**: Each agent/IDE gets its own subfolder with matching structure:
  ```
  ~/.agent-switch-tools/
  ├── device.json           ← global device identity
  ├── claude/               ← Claude Code data
  │   ├── meta.json
  │   └── profiles/{email}/
  ├── cursor/               ← Cursor IDE data
  │   └── profiles/{email}/
  ├── windsurf/             ← Windsurf IDE data
  │   └── profiles/{email}/
  └── antigravity/          ← Antigravity IDE data
      └── profiles/{email}/
  ```
- **Automatic migration**: On first launch of v1.0.10, the app transparently migrates data from all legacy locations (`~/.claude/.claude-tools/`, `~/.claude-tools/`, flat `~/.claude/`). No manual steps required.

### 4. Improved Tray Menu

- **Multi-agent sections**: The tray menu now shows separate sections for Claude Code and each installed IDE — each with its own active account and quick-switch list.
- **Live detection**: Only installed IDEs appear in the tray; uninstalled ones are hidden automatically.

---

_v1.0.10 transforms the app from a single-purpose Claude Code tool into a universal AI coding agent account manager. One app, every agent._
