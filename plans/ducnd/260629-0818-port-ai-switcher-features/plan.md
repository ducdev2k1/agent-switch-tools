# Plan: Port 3 Feature từ ai-switcher

**Created:** 2026-06-29 08:18
**Status:** Pending
**Branch:** main
**Mode:** PORT idiomatic (xia) — risk LOW, đã qua challenge gate

## Goals

Port 3 feature từ [ai-switcher](https://github.com/hoangpm96/ai-switcher) (Tauri macOS) sang dự án này (cross-platform), viết lại idiomatic theo pattern hiện có:

1. **Tray hiển thị % quota + plan** — nhãn `{name} · 96% · Plus` trên menu tray
2. **Cost analytics + date range** — dashboard chi phí/usage theo ngày, lọc 7/30/90/All
3. **Scheduled priming** (Claude Code only) — neo cửa sổ reset 5h vào giờ tự chọn, chỉ chạy khi app mở

## Context Links

- Source clone: `scratchpad/ai-switcher` (prime.rs, tray.rs, usage.rs, pricing.rs, models.rs, UsageView.tsx, AutoSessionView.tsx)
- Tray local: [tray.rs](../../../src-tauri/src/tray.rs)
- Quota worker: [quota_refresh_worker.rs](../../../src-tauri/src/quota_refresh_worker.rs)
- Session logs (nguồn cost data): [session_usage_commands.rs](../../../src-tauri/src/commands/session_usage_commands.rs)
- Claude OAuth/quota: [oauth.rs](../../../src-tauri/src/modules/providers/claude_cli/oauth.rs), [quota.rs](../../../src-tauri/src/modules/providers/claude_cli/quota.rs)
- Types FE: [types.ts](../../../src/lib/types.ts)

## Decisions (đã chốt)

| # | Quyết định |
|---|---|
| 1 | Priming trigger = tokio scheduler tick, **app phải mở** (CẮT wake/daemon macOS) |
| 2 | Priming chỉ **Claude Code** |
| 3 | Gửi "hi" + confirm = tái dùng `ensure_fresh_token` + `fetch_anthropic_usage` |
| 4 | Tray label copy y hệt `name · % · plan` |
| 5 | Nguồn cost data = tái dùng `parse_session_logs` (Claude `~/.claude/projects`) |
| 6 | Bảng giá = port LiteLLM, cache 24h, fallback stale offline |
| 7 | Chart UI = **recharts** + shadcn |

## Phases

### Phase 1: Tray % quota + plan
Inject `name · % · plan` vào `build_tray_menu`, refresh tray khi `quota-updated`. Blast radius ~0.
→ [phase-01-tray-quota-display.md](phase-01-tray-quota-display.md)

### Phase 2: Cost analytics — backend
Mở rộng parse logs lấy token breakdown, port bảng giá LiteLLM, tính cost, gom theo ngày/model, command `get_usage(range_days)`.
→ [phase-02-cost-analytics-backend.md](phase-02-cost-analytics-backend.md)

### Phase 3: Cost analytics — UI
Page/section mới dùng recharts: bar chart cost theo ngày, bảng model + session, stat tiles, range buttons, price-status badge.
→ [phase-03-cost-analytics-ui.md](phase-03-cost-analytics-ui.md)

### Phase 4: Scheduled priming — backend
`prime_account` (gửi hi + confirm anchored), scheduler tick trong quota worker, state `auto_prime` qua store, commands set/prime-now/log/stats.
→ [phase-04-scheduled-priming-backend.md](phase-04-scheduled-priming-backend.md)

### Phase 5: Scheduled priming — UI
Port `AutoSessionView` sang shadcn: time input + toggle per profile, Apply-all, Prime now, log viewer, stats table.
→ [phase-05-scheduled-priming-ui.md](phase-05-scheduled-priming-ui.md)

## Dependencies

- Phase 1 độc lập (ship ngay được)
- Phase 3 phụ thuộc Phase 2 (cần `UsageReport`)
- Phase 5 phụ thuộc Phase 4 (cần commands priming)
- Phase 2/4 độc lập nhau → có thể song song

## Success Criteria

- Tray hiện `% quota + plan` cho mỗi profile, cập nhật khi quota refresh
- Tab/section Usage hiển thị cost theo ngày (recharts), lọc range, badge giá Live/Saved/Hidden
- Đặt giờ prime cho profile Claude → app mở đến giờ tự gửi "hi", anchor window, ghi log; có per-day guard
- Cross-platform: không gọi pmset/launchd/caffeinate ở bất kỳ đâu
- Mọi file < 200 dòng, không reference plan/finding trong comment code

## Rollback Strategy

- Mỗi phase = commit riêng theo conventional commit → revert độc lập
- Phase 1: chỉ sửa `tray.rs` → revert = đổi lại `account_label`
- Phase 2-3: file mới (usage report, pricing, usage view) → gỡ command + ẩn UI entry
- Phase 4-5: scheduler tick có feature flag (chỉ chạy khi có account `enabled`) → tắt = không account nào enable; state `auto_prime` độc lập, xóa key trong store

## Unresolved Questions

1. Cost analytics: token trong Claude JSONL là số thật hay estimate? (ai-switcher mark `estimate: true`). → hiển thị badge "ước tính" nếu cần, xác nhận khi đọc `parse_single_session`.
2. UI Usage đặt ở đâu: tab mới trên dashboard hay trang riêng trong settings? → đề xuất tab mới cạnh "Claude Code"/IDE tabs.
