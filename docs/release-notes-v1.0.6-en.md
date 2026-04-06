# Claude Tools v1.0.6 Release Notes

Welcome to the **v1.0.6** update of **Claude Tools**. This release delivers critical refinements to the quota tracking system, ensuring more accurate and real-time usage data for your active profiles.

## What's New?

### 1. Enhanced Real-Time Quota Tracking

- **Context-Aware Fetching**: The application now distinguishes between active and stored profiles when fetching usage statistics.
- **Improved Accuracy**: For the currently active profile, the system now prioritizes live session credentials, providing bit-perfect accuracy of your remaining Anthropic credits.

### 2. Frontend & Hook Optimizations

- **`useProfileUsage` Refinement**: The hook has been updated to support `isActive` state synchronization, ensuring the UI always reflects the most current data for the account actually in use.
- **`ProfileCard` Integration**: The profile display component now leverages the new backend parameters to provide a more reliable status overview.

### 3. Backend (Rust) Robustness

- **Optimized Path Handling**: Refined the logic for locating credentials, reducing file system latency when switching between multiple accounts.
- **Parameter Validation**: Internal commands now feature stricter type-checking for optional parameters, improving overall app stability.

### 4. Version Updates

- Bumped internal version markers to **1.0.6** across the application core and desktop configuration.

---

_The v1.0.6 update focuses on the "Last Mile" of accuracy, ensuring that what you see in the dashboard exactly matches the data used by the Claude Code CLI during your active sessions._
