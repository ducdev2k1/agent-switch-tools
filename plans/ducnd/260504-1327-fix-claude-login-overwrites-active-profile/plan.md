---
name: Fix Claude login overwrites active profile
date: 2026-05-04
status: completed
priority: high
type: bugfix
blockedBy: []
blocks: []
---

# Fix: Login tài khoản Claude mới đè account đang active

**Created:** 2026-05-04
**Status:** Draft
**Priority:** High
**Branch:** `fix/claude-login-preserves-previous-account`

---

## Problem Statement

**Repro:**
1. App đang quản lý profile A (email_a@gmail.com), `meta.active_profile_name = "email_a@gmail.com"`
2. User chạy `claude /login` ngoài app → CLI ghi mới `~/.claude/.credentials.json` + `~/.claude.json` cho tài khoản B (email_b@gmail.com)
3. User mở app, bấm "Save Current as Profile" hoặc "Switch to <profile>"
4. **Bug:** App backup credentials của B vào folder `profiles/email_a@gmail.com/` (vì đọc theo `meta.active_profile_name` cũ) → mất profile A vĩnh viễn

**Expected:** App phải phát hiện active credentials đã đổi sang account B, lưu B vào folder `profiles/email_b@gmail.com/`, giữ nguyên profile A trong `profiles/email_a@gmail.com/`.

## Root Cause

File: [src-tauri/src/commands/config_commands.rs](../../../src-tauri/src/commands/config_commands.rs)

### Bug 1: `switch_credential_profile` (line 117-128)
```rust
let current_email = meta.active_profile_name.clone()           // ← stale cached email
    .or_else(|| auth::read_oauth_from_claude_json(&home).and_then(|o| o.email_address))
    .unwrap_or_default();

if active_path.exists() && !current_email.is_empty() {
    let prof_dir = profile_dir(&profs_dir, &current_email)?;   // ← folder của account cũ
    let backup_path = prof_dir.join("credentials.json");
    let _ = std::fs::copy(&active_path, &backup_path);          // ← ghi đè credentials mới vào folder cũ
}
```

**Sai:** Ưu tiên `meta.active_profile_name` (cached) trước `oauthAccount.email_address` (thực tế hiện tại). Khi user login ngoài app, meta đã stale.

### Bug 2: `save_current_as_profile` (line 66-99)
Hàm này đã đọc đúng email từ `~/.claude.json` (oauth thực tế), nhưng nó cũng update `meta.active_profile_name = email` mà KHÔNG kiểm tra: nếu `meta.active_profile_name` cũ ≠ email mới, có nghĩa user đã login account khác → cần thông báo/log để minh bạch.

### Bug 3: `list_credential_profiles` (line 8-65)
```rust
let active_name = meta.active_profile_name.clone()
    .or_else(|| active_oauth.as_ref().and_then(|o| o.email_address.clone()))
    .unwrap_or_else(|| "Active".to_string());
```
Cùng pattern: ưu tiên meta cached. Nếu meta nói A nhưng credentials thực là B → UI hiển thị nhầm name "A" gắn với credentials của B.

## Fix Strategy

**Nguyên tắc:** `~/.claude.json` (oauthAccount.email_address) là **source of truth** cho active account. `meta.active_profile_name` chỉ là cache, phải sync lại từ source of truth mỗi khi đọc credentials hiện hành.

### Approach: Auto-detect drift + auto-save

1. **Helper mới** `detect_active_email(home)` → đọc `oauthAccount.email_address` từ `~/.claude.json`. Trả `None` nếu không có credentials.

2. **Helper mới** `reconcile_active_profile(app, home, profs_dir, claude_data)`:
   - Đọc `actual_email` từ `~/.claude.json`
   - Đọc `meta.active_profile_name`
   - Nếu `actual_email.is_some()` và khác `meta.active_profile_name`:
     - Auto-save credentials hiện tại vào `profiles/{actual_email}/` (giữ profile cũ nguyên vẹn vì khác folder)
     - Cập nhật `meta.active_profile_name = actual_email`
     - Log event "detected external login"
   - Trả về email thực tế

3. **Sửa `list_credential_profiles`:**
   - Gọi `reconcile_active_profile` trước khi list
   - Active name = `actual_email` (không fallback meta khi credentials tồn tại)

4. **Sửa `switch_credential_profile`:**
   - Trước khi backup current, gọi `reconcile_active_profile` → đảm bảo backup vào đúng folder của account đang thực sự active
   - Nếu user có drift, profile của họ được auto-save TRƯỚC khi switch

5. **Sửa `save_current_as_profile`:**
   - Đã đọc đúng email từ `~/.claude.json`. Thêm check: nếu `meta.active_profile_name` ≠ email mới → log warning (không phải error vì đây là behavior chấp nhận được sau fix)

6. **Frontend (use-profiles.ts):**
   - Không cần đổi API surface. Nhưng cần auto-refresh khi window focus (user có thể login ngoài app rồi quay lại) → trigger `load()` on `window.focus` event

## Phases

| # | Phase | Status | Effort |
|---|-------|--------|--------|
| 1 | [Backend: reconcile helper + fix backup logic](phase-01-backend-reconcile-active-profile.md) | ✅ Completed | Medium |
| 2 | [Frontend: auto-refresh on focus + drift toast](phase-02-frontend-detect-external-login.md) | ✅ Completed | Small |
| 3 | [Tests: regression cases](phase-03-tests-regression.md) | ✅ Completed | Small |

## Key Decisions

1. **Source of truth = `~/.claude.json`** (oauthAccount.email_address), KHÔNG phải `meta.active_profile_name`
2. **Auto-reconcile, không phải prompt user** — user-experience tốt hơn, không bắt user xác nhận
3. **Giữ API surface frontend nguyên** — chỉ fix backend logic + add focus listener
4. **Không animation/migration** — folder cũ giữ nguyên, folder mới tạo song song
5. **Tương thích Cursor/Antigravity** — kiểm tra IDE flow tương tự (`ide_manager.rs:save_current_profile`) đã đọc email từ DB hiện tại nên KHÔNG bị bug này. Chỉ fix Claude.

## Files to Modify

- `src-tauri/src/commands/config_commands.rs` — fix 3 hàm + thêm helper
- `src-tauri/src/modules/providers/claude_cli/auth.rs` — có thể thêm helper `read_active_email` (optional, có thể inline)
- `src/hooks/use-profiles.ts` — thêm focus listener
- `src-tauri/src/modules/providers/claude_cli/config.rs` — không đổi (helper đã đủ)

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Race condition: user switching trong app khi reconcile chạy | Reconcile chạy đồng bộ ngay đầu mỗi command, không async background |
| User có 2 accounts cùng email (multi-org) | Email là unique key — không hỗ trợ duplicate emails (limitation hiện có) |
| Profile folder name có ký tự đặc biệt từ email | `sanitize_profile_name` đã có validate, nhưng chưa thấy gọi cho Claude flow → cần check |
| Backward compat với data hiện tại | Folder cũ giữ nguyên; meta.json schema không đổi |

## Success Criteria

- [ ] User login account B ngoài app, mở app → list hiển thị B là active, A vẫn còn trong sidebar
- [ ] Folder `profiles/email_a/` không bị overwrite
- [ ] Switch từ B sang A: credentials A được restore, credentials B được save vào `profiles/email_b/` (chứ không phải `profiles/email_a/`)
- [ ] Toast/log thông báo "Detected external login: <email>"
- [ ] Tests cover 3 scenarios: normal switch, switch sau external login, save sau external login

## Unresolved Questions

1. Có cần migrate dữ liệu cũ cho user đã bị bug này không? (Khả năng dữ liệu profile A đã bị đè mất → không restore được)
2. Có cần kiểm tra `email` từ `~/.claude.json` trùng với email trong `~/.claude/.credentials.json` (claudeAiOauth.emailAddress) không? Hiện code đọc từ `~/.claude.json` (file riêng, root level `claudeAiOauth`). Có scenario nào 2 file này khác email?
3. Frontend toast message: tiếng Việt hay tiếng Anh (i18n key)?
