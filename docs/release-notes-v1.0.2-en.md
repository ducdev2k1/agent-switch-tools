# Release Notes v1.0.2

## 🚀 Features & Updates

### 1. Usage API Integration via Anthropic OAuth

- **Quota Management Checkpoint:** Successfully integrated the Rust backend natively with Anthropic's OAuth Usage API endpoints to transparently fetch profile quotas and usage allocations.
- **Smart Limit Buckets:** We now logically pull and structure data segregating it into:
  - Session Buckets: Temporary limit constraints over rolling hourly periods.
  - 7-Day General Limits: Broad weekly volume caps allocated to the account.
  - 7-Day Sonnet Limits: Model-specific operational thresholds targeting Sonnet iterations.

### 2. Usage Tracking Frontend Enhancements

- **New `UsageLimitsDisplay` Component:** Architected a modular React component exclusively purposed to ingest and display the new telemetry metadata from the backend safely.
- **User Experience (UX) Features:**
  - Added clean, color-coded Progress Bars dynamically displaying utilization rates across the active profiles.
  - Threshold alerts via contextual styling cues when nearing constraint ceilings.
  - Directly exposes cooldown countdowns natively inside the Profile Card (e.g. "Resets in 1 hour"), letting users optimally pace workloads without hitting sudden stops.
- Synchronized with `ProfileCard` bindings to offer real-time usage observability effortlessly.

---

_This milestone strictly targets user awareness—preventing sudden work interruptions by delivering intelligent telemetry alerts directly before quotas run out!_
