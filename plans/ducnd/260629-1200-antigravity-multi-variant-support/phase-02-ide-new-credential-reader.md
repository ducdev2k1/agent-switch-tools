# Phase 02 — IDE-new Credential Reader

**Context:** [plan.md](plan.md) · [findings.md](reports/findings.md)

## Overview
- **Priority:** Cao
- **Status:** ⬜ Chưa làm
- Đọc email / display name / membership / token cho **Antigravity IDE** — bản KHÔNG còn
  `antigravityAuthStatus`, chỉ có `oauthToken` + `userStatus`.

## Key Insights
- Bản gốc lấy mọi thứ từ `antigravityAuthStatus` JSON. Bản IDE mới đã bỏ key này.
- Nguồn thay thế:
  - **token (access/refresh/expiry):** parse `antigravityUnifiedStateSync.oauthToken` (proto, đã có parser — cải tiến ở Phase 4).
  - **membership/plan:** `antigravityUnifiedStateSync.userStatus` (proto base64) — đã thấy chứa "pro".
  - **email:** chưa lộ trong `userStatus` scan nhanh → 2 phương án: (a) decode JWT trong `apiKey`/access_token nếu có claim email; (b) gọi Google `userinfo` từ access_token.

## Requirements
- **Functional:** Với folder `Antigravity IDE`, hiển thị đúng email + plan + quota; refresh token được.
- **Non-functional:** Fallback an toàn nếu thiếu field (không panic, hiển thị "Unknown").

## Architecture
- Provider `AntigravityIdeProvider` (hoặc nhánh trong provider chung) override:
  - `auth_keys()` → `["antigravityUnifiedStateSync.oauthToken", "antigravityUnifiedStateSync.userStatus"]`
  - `token_key()` → `None` (token nằm trong proto, không phải JSON apiKey)
  - `extract_membership()` → parse `userStatus` proto (tái dùng `utils::extract_plan_from_proto_json`
    nếu áp được, hoặc viết proto-walk riêng).
  - `extract_email()` → thử userinfo/JWT (xem Implementation).
  - `normalize_token()` → trích access_token từ oauthToken proto.

## Related Code Files
- **Tạo/Sửa:** `modules/providers/antigravity/mod.rs` (nhánh IDE) hoặc file mới `antigravity/ide_variant.rs`
- **Sửa:** `modules/providers/antigravity/oauth.rs` (parse userStatus + email source)
- **Sửa:** `modules/providers/utils.rs` (helper proto nếu cần)
- **Tham khảo:** `modules/providers/antigravity/quota.rs`

## Implementation Steps
1. Viết hàm parse `userStatus` proto → trích plan (và email nếu có claim).
2. Trích access_token + refresh_token từ `oauthToken` (dùng parser Phase 4).
3. Email: thử decode access_token; nếu không có → gọi Google `https://www.googleapis.com/oauth2/v3/userinfo`
   với Bearer access_token (cache kết quả). Verify trên máy trước khi chốt phương án.
4. Map các field vào `IdeProfile`/quota display.
5. Test: app hiện email + plan thật cho `Antigravity IDE`.

## Todo
- [ ] Parse `userStatus` proto → plan
- [ ] Trích token từ `oauthToken`
- [ ] Xác định nguồn email (JWT vs userinfo) — verify thực tế
- [ ] Override extract_* cho biến thể IDE
- [ ] Hiển thị đúng email/plan/quota trên UI

## Success Criteria
- `Antigravity IDE` hiện email + plan + quota chính xác như bản gốc.

## Risk Assessment
- **Email không có sẵn local** → phải gọi network (userinfo). Cần rate-limit + cache + offline fallback.

## Security Considerations
- Không log access_token/refresh_token. Gọi userinfo qua HTTPS, không lưu thừa.

## Next Steps
- Phase 4 dùng chung parser token cho refresh/quota.
