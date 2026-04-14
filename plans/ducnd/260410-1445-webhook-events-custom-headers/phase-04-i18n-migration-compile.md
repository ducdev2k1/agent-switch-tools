---
title: "Phase 4: i18n + Migration + Compile Check"
status: pending
priority: P1
effort: 30m
---

# Phase 4: i18n Strings + Config Migration + Compile Check

## Overview
Thêm i18n strings cho cả EN/VI, đảm bảo migration cũ→mới smooth, và compile check.

## Related Files
- `src/locales/en.json`
- `src/locales/vi.json`

## Implementation Steps

### 1. Add i18n strings — English (`src/locales/en.json`)

Thêm vào `settings.webhook`:
```json
"events_title": "Events Subscription",
"events_description": "Choose which events trigger the webhook. Leave all unchecked for manual-only mode.",
"event_usage_report": "Usage Report (every 5 minutes)",
"event_profile_switched": "Profile Switched",
"event_profile_created": "Profile Created",
"event_profile_deleted": "Profile Deleted",
"event_app_startup": "App Startup",
"custom_headers": "Custom Headers",
"custom_headers_hint": "Add custom HTTP headers (e.g., Bearer token, API gateway auth). Max 10 headers.",
"add_header": "Add Header",
"headers_limit": "Maximum 10 custom headers"
```

Xóa strings cũ không dùng nữa:
```
"trigger_mode", "trigger_manual", "trigger_startup", "trigger_on_change"
```

### 2. Add i18n strings — Vietnamese (`src/locales/vi.json`)

```json
"events_title": "Đăng ký sự kiện",
"events_description": "Chọn sự kiện nào sẽ kích hoạt webhook. Bỏ chọn tất cả để chỉ gửi thủ công.",
"event_usage_report": "Báo cáo sử dụng (mỗi 5 phút)",
"event_profile_switched": "Chuyển đổi hồ sơ",
"event_profile_created": "Tạo hồ sơ mới",
"event_profile_deleted": "Xóa hồ sơ",
"event_app_startup": "Khởi động ứng dụng",
"custom_headers": "Headers tùy chỉnh",
"custom_headers_hint": "Thêm HTTP headers tùy chỉnh (ví dụ: Bearer token, auth gateway). Tối đa 10 headers.",
"add_header": "Thêm Header",
"headers_limit": "Tối đa 10 headers tùy chỉnh"
```

Xóa strings cũ tương ứng.

### 3. Update Sample Payload

Trong `buildSamplePayload()`, update event field để hiển thị dynamic:
```typescript
event: 'usage_report',  // keep as example, already correct
```

Thêm custom_headers note trong sample nếu cần — không cần vì headers không nằm trong payload body.

### 4. Compile Check

```bash
# Rust check
cd src-tauri && cargo check

# TypeScript check
cd .. && pnpm tsc --noEmit

# Dev build smoke test
pnpm tauri:dev
```

## Todo
- [ ] Add EN i18n strings
- [ ] Add VI i18n strings
- [ ] Remove deprecated trigger_mode i18n strings
- [ ] Run `cargo check` — fix any Rust errors
- [ ] Run `pnpm tsc --noEmit` — fix any TS errors
- [ ] Smoke test UI in dev mode

## Success Criteria
- Both locales render correctly
- No missing translation keys in console
- `cargo check` + `tsc --noEmit` pass
- Old configs with `triggerMode` auto-migrate on load
