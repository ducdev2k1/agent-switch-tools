---
phase: 4
title: "Scheduled priming — backend"
status: pending
priority: P2
effort: "1.5-2d"
dependencies: []
---

# Phase 4: Scheduled priming — backend (Claude Code only)

## Overview
Logic neo cửa sổ reset 5h: gửi "hi" tới Anthropic, xác nhận anchored, chạy theo lịch (giờ tự chọn) trong tokio tick. Cross-platform, chỉ chạy khi app mở. KHÔNG port wake/daemon macOS.

## Requirements
- Functional: đặt giờ HH:MM + enable per profile Claude; đến giờ (app mở) tự gửi "hi" và xác nhận window anchored; per-day guard tránh prime trùng; `prime_now` thủ công; log + stats.
- Non-functional: reuse oauth/quota có sẵn; overlap guard tránh prime chồng; state bền qua tauri store; cross-platform (không pmset/launchd/caffeinate).

## Architecture
- **Prime core** `priming/prime.rs`: `prime_account(account_id) -> PrimeResult`. 4 bước (port từ source prime.rs, bỏ Codex):
  1. Token: `ensure_fresh_token` (auto refresh nếu sắp hết hạn).
  2. Classify window: đọc `fetch_anthropic_usage` → Primeable / Anchored / Ambiguous. Nếu Anchored → HOLD, không prime.
  3. Send "hi": POST `/v1/messages` Haiku, `max_tokens=1` (reqwest, dùng token). Retry 1 lần (tunable).
  4. Confirm: poll tối đa ~8 lần × 10s; success khi `reset_at` dời sang mốc tương lai mới hoặc `is_active` flip.
- **Scheduler**: thêm tick vào `quota_refresh_worker` (hoặc worker mới `priming/scheduler.rs`) chạy mỗi 60s: với mỗi account `enabled`, nếu giờ hiện tại ≥ `time` và `last_primed_date != today` → prime. `AtomicBool` overlap guard.
- **State** `priming/store.rs`: `auto_prime: Map<account_id, AutoPrimeSetting{enabled, time, last_primed_date, last_result}>` qua tauri plugin-store. Log dạng dòng (`auto-prime.log`) + stats theo ngày.
- **Commands**: `set_auto_prime(account_id, enabled, time)`, `set_auto_prime_all(time, enabled)`, `prime_now(account_id) -> PrimeResult`, `get_auto_prime_log() -> String`, `get_auto_prime_stats() -> Vec<DayStat>`.

## Related Code Files
- Create: `src-tauri/src/priming/prime.rs` (prime core + send hi + confirm)
- Create: `src-tauri/src/priming/scheduler.rs` (tick logic) — hoặc nhúng vào quota worker
- Create: `src-tauri/src/priming/store.rs` (state + log + stats)
- Create: `src-tauri/src/commands/priming_commands.rs` (5 commands)
- Modify: `src-tauri/src/lib.rs` (register commands + spawn scheduler tick)
- Reuse: `modules/providers/claude_cli/oauth.rs::ensure_fresh_token`, `quota.rs::fetch_anthropic_usage`
- Read source: `scratchpad/ai-switcher/src-tauri/src/{prime,app_state}.rs` (BỎ wake.rs)

## Implementation Steps
1. Đọc source `prime.rs` nắm 4 decision point + cách classify window; map sang `fetch_anthropic_usage` local.
2. Viết `priming/store.rs` (load/save auto_prime qua store, append log, tính stats).
3. Viết `priming/prime.rs` (`prime_account`: token → classify → send hi → confirm). Tách hàm < 200 dòng.
4. Viết scheduler tick (60s): duyệt account enabled, check giờ + per-day guard, overlap guard, gọi prime.
5. Viết `priming_commands.rs` 5 commands; register lib.rs; spawn tick lúc startup.
6. `cargo check` + test `prime_now` trên 1 profile thật (xác nhận window dời).

## Success Criteria
- [ ] `prime_now` gửi "hi" thành công và confirm window anchored (hoặc HOLD nếu đang Anchored)
- [ ] Scheduler đến giờ tự prime đúng 1 lần/ngày/profile (per-day guard)
- [ ] State `auto_prime` lưu/đọc đúng qua store; log + stats ghi nhận
- [ ] KHÔNG có lệnh pmset/launchd/caffeinate/osascript nào trong code
- [ ] Overlap guard chặn prime chồng; `cargo check` pass

## Risk Assessment
- App đóng → không prime (đã thống nhất cắt wake). Mitigations: kết hợp autostart (Phase 5 UI nhắc), hiển thị rõ "chỉ chạy khi app mở".
- Gửi "hi" tốn 1 token thật + có thể đụng rate limit → retry thưa, classify trước để không prime vào window đang chạy.
- Confirm poll lâu (~90s) → chạy async, không block; ghi last_result=failed nếu hết budget.
