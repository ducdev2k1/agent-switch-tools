# Phase 2: Session-Safe Switching

**Priority:** High | **Status:** Pending | **Depends on:** Phase 1

## Overview

Ensure switching credentials doesn't break user's working session. Detect running Claude Code processes, warn user, guide them through safe switch flow.

## Key Insights

- Claude Code caches OAuth tokens in memory — file swap doesn't affect running instance
- Session data (`~/.claude/projects/`, `sessions/`) is independent of credentials
- After file swap, user needs to start a new Claude Code session to use new account
- Running instance will continue using old tokens until they expire or get 401

## Related Files

**Modify:**
- `src-tauri/src/commands/config_commands.rs` — add process detection
- `src/pages/dashboard.tsx` — switch confirmation dialog with session warning
- `src/hooks/use-profiles.ts` — return process status with switch result

**Create:**
- `src/components/switch-confirmation-dialog.tsx` — warning dialog before switch

## Implementation Steps

### 1. Detect Running Claude Code Process

In Rust backend, check if `claude` process is running:

```rust
#[tauri::command]
pub async fn is_claude_running() -> Result<bool, String> {
    let output = std::process::Command::new("pgrep")
        .args(["-f", "claude"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(output.status.success())
}
```

### 2. Switch Confirmation Dialog

Before switching, show dialog:

**If Claude Code is running:**
> "Claude Code is currently running. Switching credentials will NOT affect the active session.
> The new account will be used when you start a new Claude Code session.
> Your project context, history, and settings will be preserved."
>
> [Switch Anyway] [Cancel]

**If NOT running:**
> "Switch to profile [name]? Your session data will be preserved."
>
> [Switch] [Cancel]

### 3. Post-Switch Notification

After successful switch, show toast:
- "Switched to [name]. New sessions will use this account."
- If Claude was running: "Restart Claude Code to use new credentials."

### 4. Switch Result Type

```rust
#[derive(Serialize)]
pub struct SwitchResult {
    pub success: bool,
    pub claude_was_running: bool,
    pub target_was_expired: bool,
    pub message: String,
}
```

Frontend displays appropriate feedback based on result fields.

## Todo

- [ ] Add `is_claude_running()` command in Rust
- [ ] Create `switch-confirmation-dialog.tsx` component
- [ ] Update switch flow: check process → confirm → swap → notify
- [ ] Add post-switch toast notifications
- [ ] Register `is_claude_running` in `lib.rs`

## Success Criteria

- User sees warning when Claude Code is running during switch
- Session data fully preserved after switch
- Clear post-switch guidance on what to do next
- No data loss in any switch scenario
