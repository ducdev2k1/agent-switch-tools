# Antigravity Multi-Variant Support

> Hỗ trợ đầy đủ (switch + quota + refresh token) cho 3 biến thể Antigravity:
> **Antigravity** (desktop gốc), **Antigravity IDE** (mới), **Antigravity CLI**.

## Bối cảnh

Google đã tách Antigravity thành 3 sản phẩm độc lập, mỗi cái lưu credentials khác nhau.
App hiện chỉ hỗ trợ đúng 1 (bản gốc `~/.config/Antigravity`) và **vỡ hoàn toàn** với 2 bản mới.

| Biến thể | `nameLong` | Nguồn credentials | Trạng thái app |
|---|---|---|---|
| Antigravity (desktop) | `Antigravity` | `~/.config/Antigravity/.../state.vscdb` — `antigravityAuthStatus` + `oauthToken` | ✅ OK |
| Antigravity IDE | `Antigravity IDE` | `~/.config/Antigravity IDE/.../state.vscdb` — **chỉ** `oauthToken` + `userStatus` | ❌ Vỡ |
| Antigravity CLI | `Antigravity CLI` | `~/.gemini/antigravity-cli/antigravity-oauth-token` (JSON file) | ❌ Chưa có |

Chi tiết phát hiện (đã verify trên máy): xem `reports/findings.md`.

## Phases

| # | Phase | Trạng thái | Mô tả |
|---|-------|-----------|-------|
| 1 | [Variant architecture](phase-01-variant-architecture.md) | ✅ Done | Tách 3 IdeType + trừu tượng hóa CredentialSource (vscdb vs json-file) |
| 2 | [IDE-new credential reader](phase-02-ide-new-credential-reader.md) | ✅ Done | Đọc token bản IDE mới + email qua Google userinfo (cache) |
| 3 | [CLI support](phase-03-cli-support.md) | ✅ Done | Reader/writer file JSON CLI + refresh Google OAuth (ISO expiry) |
| 4 | [Refresh & quota unification](phase-04-refresh-quota-unification.md) | ✅ Done | Parser oauthToken dò theo sentinel key + refresh/quota cho cả 3 biến thể |
| 5 | [UI / i18n / docs](phase-05-ui-i18n-docs.md) | ✅ Done | UI data-driven tự hiển thị 3 biến thể; README + .env.example cập nhật |

> **Build verify:** `cargo check` ✅ · `tsc --noEmit` ✅ · `vite build` ✅
> **Còn cần test thực tế (cần OAuth client secret từ CI):** refresh token + userinfo end-to-end cho IDE-new/CLI.

## Nguyên tắc thiết kế

- **KISS/DRY**: tái dùng tối đa luồng `IdeProvider` + switch/profile hiện có. Chỉ trừu tượng hóa
  đúng chỗ khác biệt: **nguồn credentials** (vscdb vs file JSON).
- **Detect động bằng `product.json`**: `nameLong`/`dataFolderName` là nguồn sự thật cho tên + path,
  thay vì hardcode rải rác.
- **Không phá vỡ bản Antigravity gốc** đang chạy tốt — chỉ mở rộng.
- **Refresh dùng Google OAuth** (đã có sẵn `antigravity::oauth::refresh_access_token`) cho cả 3.

## Phụ thuộc chính

- `src-tauri/src/modules/providers/mod.rs` — `IdeType` enum + `IdeProvider` trait
- `src-tauri/src/modules/providers/antigravity/` — provider + oauth + quota
- `src-tauri/src/modules/core/` — `sqlite_auth`, `ide_manager`, `path_helpers`
- `src-tauri/src/commands/` — `ide_commands`, `quota_commands`
- `src/` — frontend list IDE + profile cards + i18n

## Rủi ro lớn nhất

1. **CLI không phải vscdb** → phá giả định "mọi IDE = vscdb" trong `sqlite_auth`/`ide_manager`.
   → Phase 1 giải quyết bằng abstraction `CredentialSource`.
2. **OAuth client_id/secret** cho CLI có thể khác IDE (CLI dùng `~/.gemini`, có thể là Gemini client).
   → Phase 3 cần verify; fallback dùng client hiện có.
3. **Switch CLI account** = swap 1 file token (đơn giản hơn vscdb nhưng cần đảm bảo CLI không chạy).
