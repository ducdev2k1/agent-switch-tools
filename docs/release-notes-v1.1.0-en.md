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

If Claude Code was running at the time, the message tells you to **restart Claude Code**. This matters: switching only rewrites the credential, and a running `claude` session keeps using the old one until it restarts. The app does **not** kill Claude Code for you — doing so could destroy work in progress.

### 4. When every account is already spent

Nothing is switched, and you get a **single** notice instead of one every few minutes. The notice resets as soon as an account drops back below the threshold.

### 5. History of automatic switches

The Auto Switch tab records every automatic switch: when it happened, which account it left, which it moved to, and the usage that triggered it. Useful when an account changed and you want to know why.

---

## Notes

- Applies to **Claude Code** only. Cursor, Windsurf and Antigravity are not covered in this release.
- At least **two saved accounts** are required for the rule to switch anything. With a single account it can only notify you that the account is spent.
- Quota is refreshed every 5 minutes, so a switch can land up to 5 minutes after you actually cross the threshold.
- The cooldown is stored on disk, so closing and reopening the app does not reset it.

_v1.1.0 turns manual quota watching into something the app does for you — and it always tells you what it did._
