---
title: "Webhook Events Subscription & Custom Headers"
description: "Add event subscription checkboxes and custom headers key-value pairs to webhook config"
status: in_progress
priority: P1
effort: 4h
branch: main
tags: [webhook, events, custom-headers, config, ui]
created: 2026-04-10
blockedBy: []
blocks: []
---

# Webhook Events Subscription & Custom Headers

## Goal

Enhance webhook configuration with:
1. **Events Subscription** — user chọn events cụ thể trigger webhook (thay vì bắn mọi thứ)
2. **Custom Headers** — user thêm header key-value tùy ý (Bearer token, custom auth, firewall bypass)

## Current State

- Webhook đã có: URL, enabled toggle, apiKey (HMAC), secret (Bearer), triggerMode, includeCredentials, includeSessionUsage
- `triggerMode` hiện tại: `manual` | `on_startup` | `on_change` — quá đơn giản, không cho chọn event cụ thể
- Không có custom headers — chỉ hardcode X-Device-Id/X-Timestamp/X-Signature hoặc Authorization Bearer

## Approach

### Events Subscription
Thay thế `triggerMode` đơn giản bằng hệ thống events subscription:

| Event | Trigger | Mô tả |
|-------|---------|-------|
| `usage_report` | Quota refresh worker (mỗi 5 phút) | Báo cáo usage định kỳ |
| `profile_switched` | User chuyển profile | Thông báo đổi account |
| `profile_created` | User thêm profile mới | Thông báo tạo account mới |
| `profile_deleted` | User xóa profile | Thông báo xóa account |
| `app_startup` | App khởi động (sau 15s) | Ping khi app bật |

User chọn subscribe events nào → chỉ events đó mới trigger webhook.
Giữ backward compat: `triggerMode: 'on_change'` → migrate sang `subscribedEvents: ['usage_report']`.

### Custom Headers
Mảng key-value pairs `{ key: string, value: string }[]` — user tự thêm/xóa.
Được gửi kèm mọi webhook request.

## Phases

| # | Phase | Status | Effort |
|---|-------|--------|--------|
| 1 | [Backend: Add custom_headers + event param to Rust command](phase-01-backend-events-headers.md) | pending | 1h |
| 2 | [Frontend: Types + Config hook + Sender hook](phase-02-frontend-types-hooks.md) | pending | 1h |
| 3 | [UI: Events checkboxes + Custom Headers editor](phase-03-ui-events-headers.md) | pending | 1.5h |
| 4 | [i18n + Migration + Compile check](phase-04-i18n-migration-compile.md) | pending | 30m |

## Key Decisions
- Events subscription dùng `string[]` cho flexibility — dễ thêm event mới sau này
- Custom headers là `{key, value}[]` — render dynamic form với add/remove buttons
- Giữ `triggerMode` trong store migration nhưng UI chỉ hiện events subscription
- Rust command nhận `event_type: String` để ghi vào payload `"event"` field
- Custom headers inject trước HMAC/Bearer headers (user headers không override security headers)
- Limit tối đa 10 custom headers để tránh abuse
