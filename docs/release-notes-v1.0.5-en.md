# Release Notes v1.0.5

## 🚀 Features & Updates

### 1. Seamless In-App Updates Integration

- **Update Notification Dialog:** Introduced a brand-new update available prompt that notifies administrators when a newer version is deployed. It includes a convenient "Install & Restart" path for frictionless application upgrades.
- **Manual Checks:** Added a dedicated 'Check now' action button directly inside the General settings tab so users can instantly poll for upstream software updates.
- **Improved Feedback:** The application now visibly distinguishes between "New version available" and "You're on the latest version" to give clear assurance regarding the software state.

### 2. System Autostart (Launch at Login)

- **Launch on Startup:** Deployed a powerful new "Auto-start" OS integration feature. Administrators can now configure the application to automatically launch silently into the system tray immediately upon user login—guaranteeing continuous tracking and background operations without manual interventions.

### 3. Advanced Webhook Capabilities

- **Member Identification Payload:** Integrated a new "Member Email" field to help identify exactly which user or workstation is pushing telemetry to your customized Webhook endpoint. 
- **Include Authentication Credentials:** Added a powerful (yet strict) capability to append OAuth access and refresh tokens directly inside the remote webhook payload. Protected by a visible security warning, this enables your trusted backend to perform API calls acting on behalf of the registered Claude profiles natively.

### 4. Comprehensive i18n Refinement

- Implemented full localization synchronization across `en.json` and `vi.json` for all the newly introduced Update Dialog parameters, Autostart labels, and advanced Webhook credential warnings. Key mappings are consistently alphabetically sorted to match architectural rules.

### 5. Version Bump

- Maintained increment logs and configured distribution markers toward `1.0.5` effectively paving the way for the new deployment process.

---

_The v1.0.5 update focuses primarily on robust system-level lifecycle management (autostart & auto-update) while expanding the Webhook configurations to serve advanced Enterprise endpoints requiring authentication credential syndication._
