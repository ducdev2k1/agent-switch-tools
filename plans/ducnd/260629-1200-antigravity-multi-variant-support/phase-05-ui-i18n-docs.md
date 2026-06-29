# Phase 05 — UI / i18n / Docs

**Context:** [plan.md](plan.md) · [findings.md](reports/findings.md)

## Overview
- **Priority:** Trung bình
- **Status:** ⬜ Chưa làm
- Hiển thị 3 biến thể Antigravity tách biệt trên UI, thêm nhãn/badge, cập nhật i18n + docs.

## Key Insights
- Frontend list IDE theo `list_installed_ides` → tự có 3 mục khi backend trả về (Phase 1).
- Cần phân biệt trực quan: "Antigravity", "Antigravity IDE", "Antigravity CLI" (icon/badge khác nhau).
- Chuỗi hiển thị cần thêm vào `src/locales/en.json` + `vi.json`.

## Requirements
- **Functional:** 3 biến thể hiển thị rõ ràng, không nhầm lẫn; profile card + quota + nút refresh hoạt động.
- **Non-functional:** Nhất quán theme/i18n hiện có.

## Architecture
- Tái dùng component IDE/profile hiện có; chỉ bổ sung mapping label/badge theo `ide_type`.
- Badge "CLI" cho biến thể CLI; giữ icon Antigravity cho cả 3 (hoặc biến thể nhẹ).

## Related Code Files
- **Sửa:** component list IDE + profile card (trong `src/components/`)
- **Sửa:** `src/lib/types.ts` (nếu thêm ide_type mới)
- **Sửa:** `src/locales/en.json`, `src/locales/vi.json`
- **Sửa:** `docs/` — `README.md` bảng "Supported Agents", release notes, `docs/codebase-summary.md` nếu có

## Implementation Steps
1. Map `ide_type` → label/badge/icon cho 3 biến thể.
2. Thêm chuỗi i18n (en + vi).
3. Đảm bảo profile card/quota/refresh render đúng cho từng biến thể.
4. Cập nhật README "Supported Agents" + Data Location; viết release notes mới.
5. `pnpm build` (tsc) + `pnpm lint` xanh.

## Todo
- [ ] Label/badge/icon cho 3 biến thể
- [ ] i18n en + vi
- [ ] Render profile/quota/refresh đúng
- [ ] Cập nhật README + docs + release notes
- [ ] `pnpm build` + `pnpm lint` xanh

## Success Criteria
- UI hiển thị 3 biến thể rõ ràng; thao tác switch/quota/refresh hoạt động; docs phản ánh đúng.

## Risk Assessment
- **Nhầm lẫn 2 bản IDE** → label theo `nameLong` chính xác giảm rủi ro.

## Security Considerations
- Không hiển thị token/refresh_token trên UI.

## Next Steps
- Hoàn tất → `/ck:test` + `/ck:code-review` trước khi ship.
