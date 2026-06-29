---
phase: 2
title: "Cost analytics — backend"
status: pending
priority: P2
effort: "1-1.5d"
dependencies: []
---

# Phase 2: Cost analytics — backend

## Overview
Mở rộng parse session logs để lấy token breakdown, port bảng giá LiteLLM, tính cost, gom theo ngày/model, expose command `get_usage(range_days)`.

## Requirements
- Functional: trả `UsageReport` cho Claude Code gồm tổng cost, cost hôm nay, breakdown theo ngày, theo model, danh sách session; lọc theo `range_days` (7/30/90/0=all).
- Non-functional: cache giá LiteLLM tối đa 24h, fallback stale khi offline; reuse parse logic có sẵn (DRY); không block UI (background emit `usage-changed`).

## Architecture
- **Data**: tái dùng `parse_session_logs`/`parse_single_session` (session_usage_commands.rs:35) — mở rộng để thu `TokenBreakdown {input, output, cache_read, cache_creation}` per session/day/model thay vì chỉ tổng.
- **Pricing**: module mới `usage/pricing.rs` — fetch `model_prices_and_context_window.json` (LiteLLM), cache `litellm_prices.json` trong app data dir, refresh nếu cache > 24h; struct `PriceTable` + lookup model fuzzy (prefix `anthropic/`, bỏ date suffix `-YYYYMMDD`).
- **Aggregation**: module mới `usage/report.rs` — bucket `"YYYY-MM-DD|model"`, cost = `input*in + output*out + cache_read*cr + cache_creation*cc`; lọc `localToday() - range_days` bằng `chrono`.
- **Command**: `get_usage(range_days: u32) -> UsageReport` trong commands; đăng ký invoke_handler.
- **Background**: tận dụng quota worker (5 phút) emit `usage-changed` để FE refetch.

## Related Code Files
- Create: `src-tauri/src/usage/pricing.rs` (PriceTable, fetch+cache LiteLLM)
- Create: `src-tauri/src/usage/report.rs` (aggregation, cost calc)
- Create: `src-tauri/src/usage/models.rs` (structs: TokenBreakdown, DayUsage, ModelUsage, SessionUsage, ToolUsage, UsageReport)
- Create/Modify: `src-tauri/src/commands/usage_report_commands.rs` (command `get_usage`)
- Modify: `src-tauri/src/commands/session_usage_commands.rs` (mở rộng breakdown, giữ API cũ)
- Modify: `src-tauri/src/lib.rs` (register command + emit `usage-changed`)
- Read source: `scratchpad/ai-switcher/src-tauri/src/{usage,pricing,models}.rs`

## Implementation Steps
1. Đọc `parse_single_session` xác nhận token field trong JSONL (input/output/cache_read/cache_creation) và liệu là số thật hay estimate.
2. Viết `usage/models.rs` structs (port từ source models.rs, serde camelCase cho FE).
3. Viết `usage/pricing.rs`: fetch + cache 24h + fallback stale + lookup model. Trả `price_status` (Live/Saved/Hidden).
4. Viết `usage/report.rs`: gom bucket ngày/model, tính cost, lọc range_days. Tách hàm < 200 dòng/file.
5. Mở rộng parse logs lấy breakdown; giữ hàm cũ hoạt động (không phá session usage hiện có).
6. Thêm command `get_usage`, register vào lib.rs; emit `usage-changed` trong worker.
7. `cargo check` + thử `get_usage(7)` trả dữ liệu hợp lý.

## Success Criteria
- [ ] `get_usage(range_days)` trả `UsageReport` đúng cho 7/30/90/0
- [ ] Cost tính đúng theo bảng giá; offline vẫn trả (price_status=Saved) hoặc ẩn cost (Hidden)
- [ ] Cache giá refresh tối đa 1 lần/24h
- [ ] Không phá `session_usage_commands` hiện có; mỗi file < 200 dòng
- [ ] `cargo check` pass

## Risk Assessment
- Token Claude JSONL có thể là estimate → set cờ `estimate` trong report, FE hiện badge "ước tính".
- Schema LiteLLM đổi → lookup phòng thủ (Option), thiếu giá thì price_status=Hidden thay vì crash.
- Tránh đọc lại toàn bộ JSONL mỗi lần nếu chậm → cân nhắc cache cursor (port sau nếu cần, YAGNI cho bản đầu).
