---
phase: 5
title: "Scheduled priming — UI"
status: pending
priority: P2
effort: "0.5-1d"
dependencies: [4]
---

# Phase 5: Scheduled priming — UI

## Overview
Port `AutoSessionView` sang shadcn: per-profile time input + toggle enable, Apply-all, Prime now, log viewer, stats table. Khớp design tokens local.

## Requirements
- Functional: mỗi profile Claude có input giờ HH:MM + switch enable; hiển thị `time → reset ~time+5h`; nút "Apply tất cả"; nút "Prime now" (kèm confirm); xem log; bảng stats theo ngày.
- Non-functional: dùng shadcn (Card/Switch/Input/Button/Dialog); component < 200 dòng; nhắc rõ "chỉ chạy khi app mở" + gợi ý bật autostart.

## Architecture
- Hook `use-auto-prime.ts`: gọi `set_auto_prime`, `set_auto_prime_all`, `prime_now`, `get_auto_prime_log`, `get_auto_prime_stats`.
- Components:
  - `auto-session-view.tsx` (container: danh sách profile + Apply-all + log + stats) — mỏng, compose.
  - `auto-prime-row.tsx` (1 profile: time input + switch + Prime now + last_result badge).
  - `auto-prime-stats.tsx` (bảng stats theo ngày).
- Tính `reset ~time+5h` ở FE (giống source) để hiển thị.
- Entry: thêm tab "Auto Session" trong settings page hoặc tab dashboard (xác nhận chỗ đặt; đề xuất settings page cạnh Webhook).
- Banner nhắc: "Priming chỉ chạy khi app đang mở. Bật Autostart để app khởi động cùng máy." (link tới general settings).

## Related Code Files
- Create: `src/components/auto-session/auto-session-view.tsx`
- Create: `src/components/auto-session/auto-prime-row.tsx`
- Create: `src/components/auto-session/auto-prime-stats.tsx`
- Create: `src/hooks/use-auto-prime.ts`
- Modify: `src/lib/types.ts` (AutoPrimeSetting, PrimeResult, DayStat)
- Modify: `src/pages/settings-page.tsx` hoặc `src/pages/dashboard.tsx` (thêm entry)
- Read source: `scratchpad/ai-switcher/src/AutoSessionView.tsx` (tham chiếu UI)

## Implementation Steps
1. Thêm TS types (AutoPrimeSetting, PrimeResult, DayStat) vào types.ts.
2. Viết `use-auto-prime.ts` (5 invoke wrappers).
3. Viết `auto-prime-row.tsx` (time input HH:MM validate, switch, Prime now + confirm, badge last_result).
4. Viết `auto-prime-stats.tsx` + `auto-session-view.tsx` (compose + Apply-all + log viewer).
5. Thêm banner "chỉ chạy khi app mở" + link autostart.
6. Wire entry vào settings/dashboard; `pnpm build`/`tsc` pass.

## Success Criteria
- [ ] Đặt giờ + enable cho profile → lưu qua backend, reload vẫn còn
- [ ] "Prime now" gọi backend, hiện toast kết quả (success/hold/failed)
- [ ] Apply-all set cùng giờ + enable cho mọi profile Claude
- [ ] Log viewer + stats table hiển thị đúng dữ liệu backend
- [ ] Banner nhắc app-phải-mở + link autostart; component < 200 dòng; build pass

## Risk Assessment
- Người dùng kỳ vọng prime khi máy ngủ → banner nói rõ giới hạn, tránh hiểu nhầm.
- Validate giờ HH:MM ở FE trước khi gọi backend để tránh state lỗi.
