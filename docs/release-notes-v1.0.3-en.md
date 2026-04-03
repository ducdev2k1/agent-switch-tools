# Release Notes v1.0.3

## 🚀 Features & Updates

### 1. Smart & Throttled Quota Refresh System
- **Background Worker Integration:** Implemented a dedicated background thread in the Rust backend. Every 5 minutes, it silently queries the Usage API to fetch the latest quota stats without requiring the user to interact with the UI (as long as the app is running in the System Tray).
- **Intelligent Focus Refresh:** Whenever the application window regains focus, the interface automatically triggers a refresh to ensure you always see the most up-to-date figures.
- **Anti-Spam Throttling (120s):** Protected by a robust 120-second throttle mechanism. Even if you rapidly tab in and out of the application, the app strictly limits external API calls to a maximum of once per 2 minutes, entirely preventing any accidental rate-limiting locks.

### 2. Profile Card UI/UX Enhancements
- **Full-Width Usage Limits Display:** Restructured the layout for the session and weekly usage limit progress bars. The usage block now dynamically scales to the full width of the card's bottom section, expanding cleanly under the info section and right-aligned action buttons. The interface is now more seamless and visually aligned.
- **Refresh Icon Alignment:** Adjusted the usage refresh button positioning (`top-2` with an added `cursor-pointer`), aligning it precisely with the progress section's internal spacing.

### 3. Account Type Classification Fix
- **Resolved "Personal Account" mislabeling:** Fixed a bug where organizational accounts (e.g., MKT Abc) were sometimes inaccurately tagged as "Personal Account". This occurred because the `info.organizationUuid` was occasionally missing from the backend session payload.
- **Solution:** Upgraded the classification logic to additionally cross-reference the `organizationName` and `organizationUuid` extracted from the OAuth account profile, ensuring 100% accuracy when labeling an active profile as an "Organization Account".

### 4. Auto-Updater Integration
- **Auto-Update Infrastructure Setup:** Embedded the updater `pubkey` and GitHub Releases endpoint (`latest.json`) directly into the core `tauri.conf.json`, allowing the app to smoothly verify, download, and install remote updates.
- **Version Bump:** Synchronized internal package configurations to officially point to the `1.0.3` release block.

### 5. Core Tauri Build Fix (Developer)
- Fixed a compilation crash on Linux environments thrown during internal `build-script-build` execution, triggered by an unsupported process permission.
- Correctly re-mapped the outdated `process:allow-relaunch` to the valid `process:allow-restart` capability directive, restoring local codebase compilation integrity.

---

*The v1.0.3 update brings the flavor of a true professional management utility: silently intelligent, deeply polished in its UI, and precise right down to the metadata layer.*
