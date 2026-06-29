# Agent Switch Tools v1.0.13 Release Notes

**Agent Switch Tools v1.0.13** fixes upgrading across the app rename: data from the old build is now **migrated automatically**, the old build is **uninstalled** on install, and Antigravity is **temporarily hidden** while its remaining issues are worked out.

## What's new?

### 1. Automatic data migration from the old build

The app was renamed (`claude-tools` → `agent-switch-tools`) and its data root moved from `~/.claude/.claude-tools` to `~/.agent-switch-tools`. Upgrading across that rename stranded account profiles, switch history, and device identity in the old folder.

This release detects and copies that data to the new location on startup:

- Only **backfills what's missing** — never overwrites newer data.
- Merges `usageHistory` without touching the active account.
- Runs **exactly once** (marker-guarded), safe across restarts.

### 2. Old build removed on install

Because of the rename, the updater didn't recognise the old install and left **two builds running side by side** over the same data — producing an inconsistent state. This release cleans up per platform:

| Platform | Mechanism |
|----------|-----------|
| **Linux** | `.deb`/`.rpm` declare `conflicts`/`replaces`/`obsoletes` → the package manager removes the old `claude-tools` |
| **Windows** | NSIS pre-install hook runs the old "Claude Tools" uninstaller before installing |
| **macOS** | At runtime, removes `Claude Tools.app`, the old LaunchAgent login item, and orphaned WebView caches |

### 3. Antigravity temporarily hidden

Antigravity (Desktop / IDE / CLI) still has a number of issues, so it is **hidden from both the tray menu and the dashboard** for now. All of its code is kept intact and can be re-enabled once it stabilises.

---

## Notes

- On first launch of v1.0.13, old data (if any) is restored automatically — no manual steps needed.
- On Linux, if the old build was installed via `.deb`, remove the stale package once: `sudo dpkg -r claude-tools` (future `.deb` installs handle this automatically).
- Cursor and Windsurf remain hidden from the dashboard, as before.

_v1.0.13 focuses on making upgrades seamless: no data loss, no two overlapping builds._
