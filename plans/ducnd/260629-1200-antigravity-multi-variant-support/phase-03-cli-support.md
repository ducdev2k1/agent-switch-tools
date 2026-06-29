# Phase 03 — CLI Support

**Context:** [plan.md](plan.md) · [findings.md](reports/findings.md)

## Overview
- **Priority:** Trung bình-cao
- **Status:** ⬜ Chưa làm
- Hỗ trợ **Antigravity CLI**: đọc/ghi file token JSON, refresh qua Google OAuth, switch account = swap file.

## Key Insights
- CLI lưu token ở `~/.gemini/antigravity-cli/antigravity-oauth-token` — **file JSON**, không vscdb.
- Format: `{token:{access_token, token_type, refresh_token, expiry(ISO-8601)}, auth_method}`.
- `expiry` là chuỗi ISO (không phải unix millis như Claude / unix secs như IDE proto) → parser riêng.
- Refresh: cùng Google OAuth endpoint `oauth2.googleapis.com/token`; client_id/secret **cần verify**
  (có thể khác IDE vì CLI thuộc hệ Gemini).

## Requirements
- **Functional:** Hiển thị account CLI, quota, refresh token; lưu nhiều profile + switch.
- **Non-functional:** Ghi file atomic + chmod 600; không refresh khi CLI đang chạy nếu rủi ro ghi đè.

## Architecture
- `CredentialSource::JsonFile { path }` (Phase 1) cho CLI.
- `AntigravityCliProvider`:
  - `credential_source()` → JsonFile `~/.gemini/antigravity-cli/antigravity-oauth-token`.
  - đọc creds = parse JSON; email/plan: từ access_token (userinfo) hoặc field nếu có.
- Reader/Writer JSON trong `sqlite_auth` dispatcher (hoặc module mới `core/json_auth.rs`):
  - `read` → trả `HashMap`-tương-đương (access/refresh/expiry) để khớp luồng chung.
  - `write` → ghi lại file token, giữ field `auth_method`, chmod 600, atomic (tmp+rename).
- Switch CLI: copy file token saved-profile → active path (đơn giản hơn vscdb).

## Related Code Files
- **Tạo:** `modules/core/json_auth.rs` (hoặc nhánh trong `sqlite_auth.rs`)
- **Tạo:** `modules/providers/antigravity/cli_variant.rs`
- **Sửa:** `modules/shared/paths.rs` (`gemini_antigravity_cli_token_path`)
- **Sửa:** `modules/providers/antigravity/oauth.rs` (refresh nhận ISO expiry, ghi file CLI)
- **Sửa:** `modules/core/ide_manager.rs` (switch/save/list cho source JsonFile)

## Implementation Steps
1. Path resolver cho file token CLI (cross-platform: `~/.gemini/...`).
2. Reader/Writer JSON (atomic + 600), map sang cấu trúc chung.
3. Refresh CLI token: parse ISO `expiry`, gọi `refresh_access_token`, cập nhật `expiry`+`access_token`, ghi lại.
   - **Verify client_id/secret CLI** bằng 1 lần refresh thật; nếu 401 → trích client từ CLI bundle, document.
4. Profile save/switch/list cho biến thể JsonFile.
5. Quota: dùng access_token gọi API quota Antigravity (như IDE).
6. Test end-to-end: thêm/switch/refresh/quota account CLI.

## Todo
- [ ] Path resolver token CLI
- [ ] JSON reader/writer (atomic, chmod 600)
- [ ] Refresh ISO-expiry + verify OAuth client CLI
- [ ] Save/switch/list profile cho JsonFile
- [ ] Quota CLI
- [ ] Test e2e

## Success Criteria
- Account CLI hiển thị + switch + refresh + quota chạy đúng, file token không hỏng.

## Risk Assessment
- **OAuth client CLI khác** → refresh 401. Mitigation: verify sớm ở bước 3, fallback trích client.
- **Ghi đè khi CLI đang chạy** → cảnh báo/khoá như IDE (`check_ide_running`).

## Security Considerations
- Giữ chmod 600; không commit token; không log refresh_token.

## Next Steps
- Gộp refresh/quota chung ở Phase 4.
