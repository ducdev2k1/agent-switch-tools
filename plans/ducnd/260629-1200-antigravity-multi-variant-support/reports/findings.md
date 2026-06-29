# Findings — Antigravity Multi-Variant (verified trên máy ducnd, 2026-06-29)

## 1. Ba nguồn credentials

### Antigravity (desktop gốc)
- Install: `/usr/share/antigravity` → `product.json`: `nameLong=Antigravity`, `dataFolderName=.antigravity`, `urlProtocol=antigravity`
- Config: `~/.config/Antigravity/User/globalStorage/state.vscdb`
- Keys có: `antigravityAuthStatus` (JSON: `name`, `apiKey` str[260], `email`, `userStatusProtoBinaryBase64`),
  `antigravityUnifiedStateSync.oauthToken`, `antigravityUnifiedStateSync.userStatus`
- → App hiện đọc đúng biến thể này.

### Antigravity IDE (mới)
- Install: `/usr/share/antigravity-ide` → `product.json`: `nameLong=Antigravity IDE`,
  `applicationName=antigravity-ide`, `dataFolderName=.antigravity-ide`, `urlProtocol=antigravity-ide`
- Config: `~/.config/Antigravity IDE/User/globalStorage/state.vscdb`
- Keys: **KHÔNG có `antigravityAuthStatus`**. Chỉ có `antigravityUnifiedStateSync.oauthToken`
  và `antigravityUnifiedStateSync.userStatus` (proto base64, len ~8KB, chứa plan "pro").
- → App vỡ: không detect folder, không có key để lấy email/name/apiKey/plan.

### Antigravity CLI
- Binary: `/usr/bin/antigravity` → version `1.107.0`. CLI data: `~/.gemini/antigravity-cli/`
- Token file: `~/.gemini/antigravity-cli/antigravity-oauth-token` (chmod 600), JSON thuần:
  ```json
  {
    "token": {
      "access_token": "ya29.…",       // Google access token
      "token_type": "Bearer",
      "refresh_token": "1//0e…",       // Google refresh token
      "expiry": "2026-06-09T1…"        // ISO 8601 string (KHÁC: IDE dùng unix proto)
    },
    "auth_method": "consumer"
  }
  ```
- → Không phải vscdb. App chưa hỗ trợ.

## 2. oauthToken blob — thứ tự field đảo giữa các bản

Cả 2 bản IDE đều có blob chứa 2 phần: `authStateWithContextSentinelKey` (JSON `{"state":"signedIn",…}`)
và `oauthTokenInfoSentinelKey` (base64 → proto access/refresh/expiry).

- **Desktop gốc**: `oauthTokenInfoSentinelKey` đứng TRƯỚC.
- **IDE mới**: `authStateWithContextSentinelKey` đứng TRƯỚC.

→ Parser hiện tại `parse_oauth_token_blob` ([antigravity/oauth.rs:62](../../../../src-tauri/src/modules/providers/antigravity/oauth.rs#L62))
hardcode "skip f1 string, take f2 msg" → dễ gãy khi thứ tự đổi. Cần dò theo sentinel key thay vì vị trí.

## 3. Kiến trúc hiện tại (điểm cần đụng)

- `IdeType` enum {Cursor, Antigravity, Windsurf} + trait `IdeProvider` — `modules/providers/mod.rs`
- Path: `ide_app_dir(app, app_dir_name)` → `~/.config/{app_dir_name}` (Linux) — `modules/shared/paths.rs`
- DB path cố định `{app_dir}/User/globalStorage/state.vscdb` — `core/path_helpers.rs:7`
- Đọc/ghi creds: `sqlite_auth::read_ide_auth_keys` / `write_ide_auth_keys` (CHỈ vscdb)
- Switch/profile: `ide_manager::{list_profiles, save_current_profile, switch_profile, …}`
- Quota: `quota_commands::get_ide_usage` + `antigravity::quota`
- Refresh: `antigravity::oauth::{get_fresh_access_token, refresh_access_token}` (Google OAuth)
- Frontend: `list_installed_ides` → component list; `useProfileUsage`/`useTokenRefresh`

## 4. Quyết định đã chốt với user

- Phân loại: dùng `product.json nameLong` → "Antigravity", "Antigravity IDE", "Antigravity CLI" (3 mục riêng).
- Scope: **đầy đủ** — switch + quota + refresh token cho cả 3.
- Thứ tự: viết plan trước, code sau.

## Câu hỏi chưa giải quyết

1. OAuth `client_id`/`client_secret` cho CLI có giống IDE không? (CLI ở `~/.gemini` → có thể là Gemini client riêng).
   Cần test `refresh_access_token` với refresh_token của CLI bằng client hiện có; nếu 401 thì phải trích client từ CLI bundle.
2. `userStatus` proto của IDE mới: field nào chứa email? (đã thấy plan="pro"; email chưa lộ trong scan nhanh —
   có thể nằm trong `apiKey` JWT hoặc cần gọi Google `userinfo` từ access_token).
3. macOS/Windows path cho "Antigravity IDE" và CLI (`~/.gemini`) — cần verify khi build cross-platform.
