# Phase 01 — Variant Architecture

**Context:** [plan.md](plan.md) · [findings.md](reports/findings.md)

## Overview
- **Priority:** Cao nhất (nền tảng cho mọi phase sau)
- **Status:** ⬜ Chưa làm
- Tách 3 biến thể Antigravity thành 3 `IdeType` riêng và trừu tượng hóa **nguồn credentials**
  để hỗ trợ cả `state.vscdb` lẫn file JSON (CLI).

## Key Insights
- Kiến trúc hiện tại giả định **mọi IDE = state.vscdb**. CLI phá giả định đó (file JSON).
- 3 biến thể cần profile store độc lập (`~/.agent-switch-tools/{id}/profiles/`) → 3 `IdeType` là sạch nhất.
- Tên + path nên detect từ `product.json` (`nameLong`, `dataFolderName`) thay vì hardcode.

## Requirements
- **Functional:** App detect độc lập 3 biến thể; mỗi biến thể có path + cách đọc creds riêng.
- **Non-functional:** Không phá vỡ Cursor/Windsurf/Antigravity gốc; KISS/DRY.

## Architecture
1. Mở rộng `IdeType` → thêm `AntigravityIde`, `AntigravityCli` (giữ `Antigravity`).
   - `id()`: `"antigravity"`, `"antigravity-ide"`, `"antigravity-cli"`.
2. Thêm enum **`CredentialSource`**:
   ```rust
   enum CredentialSource {
       Vscdb { db_path: PathBuf },     // Cursor/Windsurf/Antigravity*/IDE
       JsonFile { path: PathBuf },     // Antigravity CLI
   }
   ```
3. Thêm method vào `IdeProvider`: `fn credential_source(&self, app) -> Result<CredentialSource, String>`.
   - Default impl trả `Vscdb` (giữ nguyên hành vi cũ cho Cursor/Windsurf).
4. `sqlite_auth::read/write_ide_auth_keys` → tách thành dispatcher theo `CredentialSource`:
   - `Vscdb` → logic hiện tại.
   - `JsonFile` → reader/writer JSON (chi tiết ở Phase 3).
5. `path_helpers::ide_db_path` → giữ cho biến thể vscdb; thêm resolver cho CLI file.
6. Detect installed: `ide_is_installed` dựa trên `credential_source` tồn tại (file/db).

## Related Code Files
- **Sửa:** `modules/providers/mod.rs` (IdeType, trait, CredentialSource)
- **Sửa:** `modules/providers/antigravity/mod.rs` (provider gốc) + **tạo** module/variant cho IDE+CLI
- **Sửa:** `modules/core/path_helpers.rs`, `modules/core/sqlite_auth.rs`, `modules/core/ide_manager.rs`
- **Sửa:** `modules/shared/paths.rs` (resolver `~/.gemini/antigravity-cli`)

## Implementation Steps
1. Thêm 2 biến thể vào `IdeType` enum + `all()` + `from_str()` + `provider()`.
2. Định nghĩa `CredentialSource` + thêm method trait (default `Vscdb`).
3. Refactor `read/write_ide_auth_keys` thành dispatch theo source (chưa cần JSON impl thật — stub trả lỗi "not yet").
4. Cập nhật `ide_is_installed` + `list_installed_ides` để 3 biến thể xuất hiện đúng.
5. Build `cargo check` xanh; Cursor/Windsurf/Antigravity gốc không đổi hành vi.

## Todo
- [ ] Thêm `AntigravityIde`, `AntigravityCli` vào `IdeType`
- [ ] Định nghĩa `CredentialSource` + trait method
- [ ] Refactor read/write auth keys theo dispatcher
- [ ] Cập nhật detect installed + list
- [ ] `cargo check` xanh, regression Cursor/Windsurf OK

## Success Criteria
- `list_installed_ides` trả về `Antigravity`, `Antigravity IDE`, `Antigravity CLI` đúng trạng thái cài đặt.
- Không có thay đổi hành vi cho Cursor/Windsurf/Antigravity gốc.

## Risk Assessment
- **Mở rộng enum lan rộng nhiều match** → dùng default trait impl + helper để giảm chỗ phải sửa.

## Security Considerations
- File token CLI chmod 600 — giữ nguyên quyền khi đọc/ghi.

## Next Steps
- Phase 2 (đọc creds IDE mới) + Phase 3 (CLI JSON impl) build trên abstraction này.
