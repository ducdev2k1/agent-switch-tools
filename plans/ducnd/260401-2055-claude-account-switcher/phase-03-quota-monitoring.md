# Phase 3: Quota & Usage Monitoring

**Priority:** Medium | **Status:** Pending | **Independent of Phase 2**

## Overview

Show credential health info per profile: subscription type, rate limit tier, token expiry countdown, usage history. Help user decide which account to switch to.

## Key Insights

- `rateLimitTier` and `subscriptionType` already in credentials — just need better display
- No public Anthropic API for checking remaining quota (as of Apr 2026)
- Token `expiresAt` is OAuth expiry, not quota — but useful for health check
- Can track local usage: when each profile was last active, how long used

## Related Files

**Modify:**
- `src-tauri/src/commands/config_commands.rs` — enrich CredentialInfo
- `src-tauri/src/commands/quota_commands.rs` — usage history tracking
- `src/components/profile-card.tsx` — display quota badges, expiry countdown
- `src/lib/types.ts` — extended types

**Modify (metadata):**
- `~/.claude/.claude-manager-meta.json` — add usage history

## Implementation Steps

### 1. Enrich Profile Display Info

Add to `CredentialInfo`:
```rust
pub struct CredentialInfo {
    // ... existing fields
    pub is_expired: bool,         // computed from expires_at
    pub expires_in_hours: Option<f64>, // hours until expiry
}
```

### 2. Usage History in Metadata

Track when each profile was last active:
```json
{
  "activeProfileName": "Work",
  "lastSwitchedAt": "2026-04-01T20:00:00Z",
  "usageHistory": {
    "Work": { "lastActiveAt": "2026-04-01T20:00:00Z", "totalActiveMinutes": 1440 },
    "Personal": { "lastActiveAt": "2026-03-31T10:00:00Z", "totalActiveMinutes": 720 }
  }
}
```

On switch: calculate duration since `lastSwitchedAt`, add to outgoing profile's `totalActiveMinutes`.

### 3. Profile Health Badges

Visual indicators on profile cards:

| Condition | Badge | Color |
|-----------|-------|-------|
| Token valid, > 24h left | "Active" | Green |
| Token expires < 24h | "Expiring Soon" | Yellow |
| Token expired | "Expired" | Red |
| Subscription: team/max | Tier badge | Blue |
| Recently used (< 1h ago) | "Recent" | Gray |

### 4. Expiry Countdown

Show relative time on profile card: "Expires in 23h", "Expired 2h ago".

Use `chrono` in Rust to compute, or compute in frontend from `expiresAt` timestamp.

### 5. Update Quota Commands

`src-tauri/src/commands/quota_commands.rs`:
- `get_profile_health(name)` — returns expiry status, subscription info, last used
- Update `get_usage_stats()` to include per-profile breakdown

## Todo

- [ ] Add `is_expired` and `expires_in_hours` to `CredentialInfo` struct
- [ ] Track usage history in metadata file (on switch events)
- [ ] Add health badges to `profile-card.tsx`
- [ ] Show expiry countdown on each profile card
- [ ] Update `quota_commands.rs` with profile health check

## Success Criteria

- Each profile card shows subscription type, tier, and expiry status
- Color-coded badges indicate credential health at a glance
- Usage history shows when each profile was last active
- User can quickly identify which account to switch to
