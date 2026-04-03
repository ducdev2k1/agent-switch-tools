# Release Notes v1.0.4

## 🚀 Features & Updates

### 1. Introduction of the Settings Page

- **New Configuration Hub:** Successfully integrated a dedicated Settings interface, effortlessly accessible directly from the main dashboard via the ⚙️ (Gear) icon.
- **Sidebar Menu Layout:** The Settings screen is meticulously structured into logical tabs via a sidebar menu: `General`, `Webhook`, and `About`.

### 2. Webhook Integrations (Usage Reporting)

- **Customizable Endpoints:** Users can now define a personalized Webhook URL Endpoint to automatically dispatch Quota and Usage Reports. This feature answers the critical need to report telemetry from multiple managed profiles to a centralized server.
- **Flexible Trigger Options:** 
  - Send reports automatically upon application startup.
  - Automatically dispatch a webhook whenever there is a shift in quota or usage metrics.
  - Trigger telemetry data manually via a standalone "Send Now" button.
- **Enhanced Security:** Added support for configuring a personalized Authentication Secret. If linked, the system wraps this token securely within the `Authorization: Bearer <Secret>` Header across all outgoing webhook requests.
- **Test Connection Utility:** Provided a built-in simulation mechanism allowing you to independently test the integrity of the connection between the application and your target endpoint prior to full deployment.

### 3. General Preferences

- **Theme Modes:** Alter visual elements directly within the Settings panel, providing seamless toggle options across Light, Dark, and System states.
- **Auto Update Management:** Embedded a switch granting explicit control over background update-check capabilities.
- **Information Central (About):** Consolidated the application version, developer credits, and open-source licensing metadata securely into the About component.

### 4. Comprehensive i18n Refinement

- Implemented 100% dictionary synchronization for the freshly added Settings Page. Translated all user-facing labels, input placeholders, and system toast alerts across the `en.json` and `vi.json` manifests—respecting precise alphabetical sorting standards.

### 5. Version Bump

- Upgraded the central distribution markers in both `package.json` and `tauri.conf.json` to logically reflect the `1.0.4` product increment.

---

_The v1.0.4 update empowers administrators with formidable data transmission features via Webhook integrations—essentially evolving the application into an orchestrator that seamlessly binds local profiles with central Data Centers._
