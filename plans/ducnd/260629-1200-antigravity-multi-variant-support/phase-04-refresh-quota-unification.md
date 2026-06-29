# Phase 04 — Refresh & Quota Unification

**Context:** [plan.md](plan.md) · [findings.md](reports/findings.md)

## Overview
- **Priority:** Cao
- **Status:** ⬜ Chưa làm
- Làm parser `oauthToken` robust với thứ tự field đảo, và đảm bảo refresh token + fetch quota
  chạy thống nhất cho cả 3 biến thể.

## Key Insights
- Parser `parse_oauth_token_blob` ([antigravity/oauth.rs:62](../../../../src-tauri/src/modules/providers/antigravity/oauth.rs#L62))
  dò theo **vị trí** field (skip f1, take f2) → gãy khi thứ tự đổi giữa desktop vs IDE
  (đã verify: desktop để `oauthTokenInfoSentinelKey` trước, IDE để `authStateWithContextSentinelKey` trước).
- Cần dò theo **sentinel key name** (`oauthTokenInfoSentinelKey`) thay vì index.

## Requirements
- **Functional:** Trích đúng access/refresh/expiry bất kể thứ tự field; refresh + quota cho cả 3 biến thể.
- **Non-functional:** Best-effort (lỗi refresh → giữ token cũ, không panic).

## Architecture
- Viết lại `parse_oauth_token_blob`: duyệt toàn bộ map entries, tìm entry có key == `oauthTokenInfoSentinelKey`,
  rồi mới decode inner base64 → proto access/refresh/expiry. Bỏ giả định vị trí.
- Tách hàm refresh dùng chung:
  - IDE (desktop + IDE-new): refresh → ghi lại proto vào vscdb (`write_ide_auth_keys`).
  - CLI: refresh → ghi lại file JSON (Phase 3).
- Quota: `get_ide_usage` (quota_commands) đã generic theo provider; đảm bảo 3 biến thể đi qua
  `ensure fresh token` trước khi gọi quota API (giống luồng Claude `resolve_claude_token`).
- Background worker quota: cân nhắc mở rộng để refresh quota IDE/CLI định kỳ (hiện worker chỉ lo Claude).

## Related Code Files
- **Sửa:** `modules/providers/antigravity/oauth.rs` (parser robust + refresh-persist theo source)
- **Sửa:** `commands/quota_commands.rs` (`get_ide_usage` đảm bảo refresh trước fetch)
- **Sửa (tùy chọn):** `quota_refresh_worker.rs` (mở rộng cho IDE/CLI)
- **Tham khảo:** `modules/core/sqlite_auth.rs::write_ide_auth_keys`

## Implementation Steps
1. Rewrite parser để dò theo sentinel key; thêm test với blob desktop + IDE (cả 2 thứ tự).
2. Hàm `ensure_fresh_ide_token(source)` → refresh nếu gần hết hạn, persist đúng nơi (vscdb/JSON).
3. `get_ide_usage` gọi ensure-fresh trước khi fetch quota.
4. (tùy chọn) worker refresh quota định kỳ cho IDE/CLI với rate-limit như hiện tại.
5. Test refresh + quota cho cả 3 biến thể.

## Todo
- [ ] Parser oauthToken dò theo sentinel key (+ test 2 thứ tự)
- [ ] `ensure_fresh_ide_token` persist theo CredentialSource
- [ ] `get_ide_usage` refresh-before-fetch
- [ ] (tùy chọn) mở rộng quota worker
- [ ] Test refresh+quota 3 biến thể

## Success Criteria
- Token hết hạn của cả 3 biến thể được tự refresh; quota hiển thị đúng; không 401 sau ~1h.

## Risk Assessment
- **Proto format đổi tiếp trong tương lai** → parser dò-theo-key bền hơn dò-theo-vị-trí.
- **Rate limit Google OAuth / quota API** → giữ skew + cache + delay giữa profiles.

## Security Considerations
- Không log token; ghi vscdb/file atomic; giữ quyền file.

## Next Steps
- Phase 5 hiển thị trạng thái + cập nhật docs.
