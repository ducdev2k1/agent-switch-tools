# Agent Switch Tools v1.0.16 Release Notes

**Agent Switch Tools v1.0.16** makes account switching faster and smoother: the confirmation dialog is gone, tray quick-switch now runs fully in the background, the account list keeps a stable order, list view is fully localized, and the UI got a cleanup (no more bright green stripe, wider default window).

## What's New?

### 1. One-click account switching — no confirmation dialog

Previously every switch (Claude Code or IDE) opened a confirmation dialog. Now **clicking an account switches immediately**:

- Applies to cards (grid), the table (list) and the tray menu.
- Important warnings are kept, just moved to post-switch notifications: if Claude Code is running, the app reminds you to restart it to use the new account.
- For IDEs: if the IDE is running at switch time, the app **auto-restarts it** as before — only the question step is gone.

### 2. Tray quick-switch runs fully in the background

Previously clicking a profile in the tray opened the app window first — and for IDEs it only opened the right tab, you still had to click the profile again. Now:

- Click a profile in the tray → **the switch happens immediately in the background**, no window pops up.
- Works even if the dashboard has **never been opened** — the tray no longer depends on the UI.
- If the app is open, the list refreshes automatically and a "Switched to..." toast appears.
- IDE accounts: after switching, the IDE is auto-restarted if it was running.

### 3. Stable account list order

Previously the active account jumped to the top of the list after every switch — card positions shuffled around, making it hard to find the account you just used. The list is now **always sorted alphabetically**; switching never reorders it. The active account is identified by its green tint and ACTIVE badge.

### 4. Fully localized list view

The list view used to show raw translation keys in its column headers. It is now fully translated:

- Column headers: **Email / Membership / Model quota / Expires at / Status / Actions**.
- Action buttons (refresh token, refresh quota, switch, delete) show tooltips on hover.

### 5. UI cleanup

- **Removed the bright green edge stripe** on the active account card — less glare, while the subtle green tint, dot and ACTIVE badge still mark it clearly.
- **Wider default window**: 1200×720 (was 900×640) — the account grid shows **3 cards per row** out of the box, no manual resizing needed.

---

## Notes

- The new window size only applies to freshly opened windows; if your OS remembers the old size, resize once and you're set.
- Since there is no confirmation step anymore, watch the post-switch notifications — if Claude Code is running, restart it to use the new account.

_v1.0.16 focuses on cutting account switching down to a single click and making the UI cleaner and more predictable._
