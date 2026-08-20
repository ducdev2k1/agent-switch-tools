# Agent Switch Tools v1.1.0 Release Notes

**Agent Switch Tools v1.1.0** adds the **Auto Switch Rule** — the app moves you to the account with the most quota left as soon as the one you are using reaches a threshold you set. Until now you had to watch the percentage yourself and switch by hand; that work now happens in the background.

## What's new?

### 1. "Auto Switch" tab in Settings

**Off by default.** Turn it on and set two values:

- **Switch Threshold** — 50–99%, default 90%. When the account in use reaches it, the app switches away.
- **Cooldown** — 5–120 minutes, default 5. Caps how often an automatic switch can happen.

The threshold is measured on Claude Code's **5-hour limit** — the one that actually blocks your work. The weekly limits (7-day, 7-day Sonnet) are deliberately ignored: an account that is out for the week cannot be rescued by switching away from it for five minutes, and that situation needs a different answer.

### 2. Target selection: whichever account has the most quota left

The app already refreshes the quota of **every** account every 5 minutes, so it knows which one is freest without making a single extra API call. When the rule fires, it moves to the account with the lowest 5-hour usage among those still below the threshold.

After a switch the app **stays** on the new account — it does not jump back when the old one's quota resets. Switching back is your call.

### 3. You always find out, even when the app is in the tray

Because the rule runs in the background, telling you about it is part of the feature, not a decoration:

- **Desktop notification** — visible even while the app is minimised to the tray.
- **In-app toast** — appears on any page, not only while the Auto Switch tab is open.
- **Tray menu** refreshes to show the new account.

This matters more than it sounds. Switching rewrites `~/.claude/.credentials.json`, the file Claude Code reads per request — so a session you are in the middle of **starts billing the new account immediately**, while the account it *displays* stays the old one until you open a new session. The running session therefore cannot tell you what happened; the notification is the only thing that can. The app does **not** kill Claude Code for you: that is unnecessary (the switch already took effect) and could destroy work in progress.

### 4. When every account is already spent

Nothing is switched, and you get a **single** notice instead of one every few minutes. The notice resets as soon as an account drops back below the threshold.

### 5. History of automatic switches

The Auto Switch tab records every automatic switch: when it happened, which account it left, which it moved to, and the usage that triggered it. Useful when an account changed and you want to know why.

### 6. The Auto Session tab no longer lags

Not part of the auto-switch feature, but it shared a root cause, so it was fixed in the same pass.

The Auto Session activity log used to be handed to the interface as **one single string**, which then rendered **every** line at once. On a machine whose log had grown to 25,000 lines, opening that tab meant shipping 2.5 MB across and building roughly 125,000 table cells — several seconds before anything appeared.

Why the log grew: a profile whose credential had been removed was still recorded by the scheduler as `skip | credentials not found` **once a minute, for weeks**. Nearly all of those 25,000 lines were noise.

All three layers are fixed:

- The scheduler passes over profiles with no credential silently, keeping their schedule in case the account comes back.
- Activity logs cap themselves: past 5,000 lines they are rewritten down to the newest 2,000. An existing oversized log is trimmed the first time you launch this version.
- The log table is paged at 100 rows, and lines are parsed before they reach the interface. The Auto Switch history table uses the same approach.

---

## Notes

- Applies to **Claude Code** only. Cursor, Windsurf and Antigravity are not covered in this release.
- At least **two saved accounts** are required for the rule to switch anything. With a single account it can only notify you that the account is spent.
- Quota is refreshed every 5 minutes, so a switch can land up to 5 minutes after you actually cross the threshold.
- No restart is needed for a switch to apply. Restart only if you want the account name shown inside a running session to match.
- The cooldown is stored on disk, so closing and reopening the app does not reset it.

_v1.1.0 turns manual quota watching into something the app does for you — and it always tells you what it did._
