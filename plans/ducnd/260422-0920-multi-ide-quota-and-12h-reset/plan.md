# Plan: Multi-IDE Quota + 12h Reset Time Display

**Created:** 2026-04-22 09:20
**Status:** Phase 1 + 3 DONE ✓ | Phase 2 (Cursor) SKIPPED per user decision
**Branch:** main

## Goals

1. **Fetch quota cho các IDE khác** (Cursor, Windsurf) — hiện tại chỉ có Claude CLI + Antigravity
2. **Claude CLI**: hiển thị thời gian reset dạng đồng hồ 12h (AM/PM) bên cạnh relative time hiện tại

## Context Links

- Research: [researcher-260422-0921-cursor-windsurf-quota-apis.md](../reports/researcher-260422-0921-cursor-windsurf-quota-apis.md)
- Backend quota: [quota_commands.rs](../../../src-tauri/src/commands/quota_commands.rs)
- Claude CLI quota: [modules/providers/claude_cli/quota.rs](../../../src-tauri/src/modules/providers/claude_cli/quota.rs)
- Antigravity quota: [modules/providers/antigravity/quota.rs](../../../src-tauri/src/modules/providers/antigravity/quota.rs)
- Frontend: [usage-limits-display.tsx](../../../src/components/usage-limits-display.tsx), [use-ide-usage.ts](../../../src/hooks/use-ide-usage.ts)

## Key Findings từ Research

| IDE | Official quota API? | Feasibility |
|-----|---------------------|-------------|
| **Claude CLI (Anthropic)** | ✅ `GET /api/oauth/usage` — đang dùng | Đã xong |
| **Antigravity (Google)** | ✅ `cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels` — đang dùng | Đã xong |
| **Cursor** | ❌ Không có public single-user API. Chỉ Enterprise Admin API | Có thể reverse-engineer nhưng fragile |
| **Windsurf** | ❌ Enterprise-only, cần service key trong body | Không khả thi cho consumer app |

**Kết luận**: Chỉ có thể integrate **Cursor** ở mức best-effort (community endpoint), Windsurf đành mark "unsupported".

## Phases

### Phase 1: Claude CLI — Reset time 12h format
Thêm absolute time (12h AM/PM format) bên cạnh relative countdown hiện tại.

**Scope**: Frontend only — `usage-limits-display.tsx`.

**Files**: [phase-01-claude-cli-12h-reset-format.md](phase-01-claude-cli-12h-reset-format.md)

### Phase 2: Cursor quota (best-effort)
Thêm Cursor provider quota via `api2.cursor.sh/auth/full_stripe_profile` (community-known).

**Scope**: Backend `modules/providers/cursor/quota.rs` + wire vào `quota_commands::get_ide_usage`.

**Caveats**:
- Token format: JWT lấy từ `cursorAuth/accessToken`, có thể cần decode/convert
- Response schema không official → phải thử nghiệm
- Có thể break khi Cursor update

**Files**: [phase-02-cursor-quota-best-effort.md](phase-02-cursor-quota-best-effort.md)

### Phase 3: Windsurf — Graceful "not supported"
Windsurf không thể lấy quota user-level → hiển thị rõ trong UI thay vì fail im lặng.

**Scope**: Frontend — detect Windsurf case, show "Quota không khả dụng".

**Files**: [phase-03-windsurf-unsupported-ui.md](phase-03-windsurf-unsupported-ui.md)

## Dependencies

- Phase 1, 2, 3 độc lập → có thể chạy song song nếu cần

## Success Criteria

- Claude CLI profile card hiển thị cả "R: 2h 15m" và "3:45 PM"
- Cursor profile card hiển thị quota (nếu API trả về) hoặc "—" nếu fail
- Windsurf profile card hiển thị "Quota không khả dụng" rõ ràng
- Không crash khi API fail; cache 120s để tránh rate limit

## Unresolved Questions (cần user confirm trước khi code)

1. **Cursor quota**: user OK với việc integrate best-effort (có thể break bất cứ lúc nào Cursor update)? Nếu không, bỏ Phase 2.
2. **Format 12h**: hiển thị kiểu nào?
   - (a) Thay thế hẳn: `Reset: 3:45 PM`
   - (b) Bổ sung: `R: 2h 15m (3:45 PM)` — khuyến nghị
   - (c) Tooltip: hover vào "2h 15m" show "3:45 PM"
3. **Windsurf**: bỏ hẳn quota card hay vẫn show nhưng với label rõ ràng?
4. **Scope khác**: có IDE nào khác cần thêm không (Zed, Continue.dev, Aider, Codeium non-Windsurf)?
