---
phase: 1
title: "Tray hiển thị % quota + plan"
status: pending
priority: P1
effort: "2-3h"
dependencies: []
---

# Phase 1: Tray hiển thị % quota + plan

## Overview
Hiển thị `{name} · {percent}% · {plan}` cho mỗi profile trên menu tray, cập nhật khi quota refresh. Feature dễ nhất, ship đầu tiên.

## Requirements
- Functional: mỗi item profile Claude trên tray hiện tên + % sử dụng (5h window) + tên gói; profile active có checkmark.
- Non-functional: không block menu handler; không thêm timer mới (tận dụng `quota-updated` đã có); fallback gọn khi chưa có quota (chỉ hiện tên).

## Architecture
- Quota worker đã emit `quota-updated:{name}` và lưu quota mỗi profile. Cần đọc quota gần nhất khi build menu.
- `build_tray_menu` (tray.rs:81) hiện chỉ dựng nhãn tên → thêm hàm `profile_tray_label(name, &Option<UsageLimits>, plan)` trả `"{name} · 96% · Plus"`.
- Percent lấy từ `UsageLimits.fiveHour.utilization` (làm tròn int). Plan lấy từ field plan trong quota payload (xác nhận tên field khi đọc `quota.rs`).
- Gọi `refresh_tray_menu` (tray.rs:72) trong listener `quota-updated` (nếu chưa có thì thêm subscribe ở nơi setup tray / lib.rs).

## Related Code Files
- Modify: `src-tauri/src/tray.rs` (thêm `profile_tray_label`, dùng trong `build_tray_menu`)
- Modify: `src-tauri/src/quota_refresh_worker.rs` hoặc `src-tauri/src/lib.rs` (trigger `refresh_tray_menu` sau khi emit `quota-updated`)
- Read: `src-tauri/src/modules/providers/claude_cli/quota.rs` (struct UsageLimits + field plan)
- Read source: `scratchpad/ai-switcher/src-tauri/src/tray.rs` (`account_label`, `build_menu`)

## Implementation Steps
1. Đọc `quota.rs` xác nhận tên field percent (`utilization`) và plan; nếu plan chưa parse, bổ sung parse từ response.
2. Viết helper `profile_tray_label` trong `tray.rs` (KISS: format string, bỏ phần quota nếu None).
3. Trong `build_tray_menu`, lấy quota gần nhất mỗi profile (từ cache worker hoặc đọc nhanh) → dùng helper cho nhãn.
4. Đảm bảo `refresh_tray_menu` được gọi sau mỗi lần quota cập nhật (subscribe `quota-updated`).
5. Build kiểm tra compile (`cargo check` trong src-tauri).

## Success Criteria
- [ ] Tray hiện `name · % · plan` cho profile có quota; chỉ `name` nếu chưa có quota
- [ ] Profile active có checkmark
- [ ] Nhãn tự cập nhật khi quota worker refresh (không cần mở lại app)
- [ ] `cargo check` pass, không cảnh báo mới nghiêm trọng

## Risk Assessment
- Rủi ro thấp. Nếu field plan không có sẵn trong payload → bỏ phần plan, chỉ hiện `name · %` (degrade gracefully).
- Tránh fetch quota đồng bộ trong handler tray (block UI) → chỉ đọc cache, không gọi network khi build menu.
