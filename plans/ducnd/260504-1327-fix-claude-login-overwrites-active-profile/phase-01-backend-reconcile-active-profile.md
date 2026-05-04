# Phase 1 — Backend: Reconcile active profile + fix backup logic

**Status:** Pending
**Priority:** Critical
**Effort:** Medium

## Context Links

- Bug analysis: [plan.md](./plan.md#root-cause)
- Source of truth: `~/.claude.json` → `claudeAiOauth.email_address`
- Existing helpers: `src-tauri/src/modules/providers/claude_cli/auth.rs:14` `read_oauth_from_claude_json`

## Overview

Fix root cause của bug: `meta.active_profile_name` bị stale khi user login ngoài app. Logic backup hiện tại đọc meta cached → ghi credentials mới vào folder profile cũ → đè account cũ.

Fix bằng cách thêm helper `reconcile_active_profile` để sync `meta.active_profile_name` với `~/.claude.json` thực tế trước mỗi thao tác đọc/ghi profile.

## Key Insights

- `oauthAccount.email_address` trong `~/.claude.json` là source of truth duy nhất cho active account
- `meta.active_profile_name` chỉ là cache để tăng tốc UI, KHÔNG được dùng làm khoá xác định folder backup
- IDE flow (Cursor/Antigravity) trong `ide_manager.rs:save_current_profile` đã đọc email từ DB hiện tại — không bị bug này. Chỉ fix Claude.

## Requirements

### Functional
- Khi `meta.active_profile_name` ≠ `oauthAccount.email_address` thực tế:
  - Auto-save credentials hiện tại vào folder `profiles/{actual_email}/`
  - Update `meta.active_profile_name = actual_email`
  - Emit event/log để frontend hiển thị toast
- Backup credentials phải dùng email từ `~/.claude.json`, KHÔNG dùng meta cached
- Switch profile: trước khi ghi đè active, đảm bảo current credentials được lưu vào đúng folder

### Non-functional
- Reconcile chạy đồng bộ (sync), không async background → tránh race
- Không phá API surface frontend hiện tại
- Backward compatible: folder cũ không bị migrate/đụng vào

## Architecture

```
list_credential_profiles ──┐
switch_credential_profile ─┼──► reconcile_active_profile() ──► oauthAccount.email_address
save_current_as_profile  ──┘                                   │
                                                                ├─► auto-save vào profiles/{email}/
                                                                └─► update meta.active_profile_name
```

## Related Code Files

**Modify:**
- `src-tauri/src/commands/config_commands.rs` — sửa 3 commands + thêm helper

**Read for context:**
- `src-tauri/src/modules/providers/claude_cli/auth.rs` — `read_oauth_from_claude_json`, `write_saved_oauth`
- `src-tauri/src/modules/providers/claude_cli/config.rs` — `read_meta`, `write_meta`, `set_file_600`
- `src-tauri/src/modules/shared/paths.rs` — `profile_dir`, `home_dir`, `claude_dir`, `profiles_dir`

**Create:** none (helper inline trong `config_commands.rs` hoặc thêm vào `auth.rs`)

## Implementation Steps

### Step 1: Thêm helper `reconcile_active_profile`

Thêm vào đầu `config_commands.rs` (private fn):

```rust
/// Sync meta.active_profile_name với oauthAccount.email_address thực tế.
/// Nếu phát hiện drift (user login ngoài app), auto-save credentials hiện tại
/// vào folder của email mới và update meta.
///
/// Returns: (actual_email, drift_detected)
fn reconcile_active_profile(
    home: &PathBuf,
    claude: &PathBuf,
    profs_dir: &PathBuf,
    claude_data: &PathBuf,
) -> Result<(Option<String>, bool), String> {
    let active_path = claude.join(".credentials.json");
    if !active_path.exists() {
        return Ok((None, false));
    }

    let oauth = match auth::read_oauth_from_claude_json(home) {
        Some(o) => o,
        None => return Ok((None, false)),
    };
    let actual_email = match &oauth.email_address {
        Some(e) if !e.is_empty() => e.clone(),
        _ => return Ok((None, false)),
    };

    let mut meta = config::read_meta(claude_data);
    let cached_email = meta.active_profile_name.clone().unwrap_or_default();

    if cached_email == actual_email {
        return Ok((Some(actual_email), false));
    }

    // Drift detected — auto-save credentials hiện tại vào folder của actual_email
    let prof_dir = crate::modules::shared::paths::profile_dir(profs_dir, &actual_email)?;
    let backup_path = prof_dir.join("credentials.json");
    std::fs::copy(&active_path, &backup_path)
        .map_err(|e| format!("Reconcile: failed to save active credentials: {}", e))?;
    config::set_file_600(&backup_path);

    auth::write_saved_oauth(profs_dir, &actual_email, &oauth)?;

    meta.active_profile_name = Some(actual_email.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    config::write_meta(claude_data, &meta)?;

    eprintln!(
        "[reconcile] Detected external login: cached={:?}, actual={}",
        cached_email, actual_email
    );

    Ok((Some(actual_email), true))
}
```

### Step 2: Sửa `list_credential_profiles`

Trước block `let active_oauth = ...`, thêm:

```rust
let (_, drift_detected) = reconcile_active_profile(&home, &claude, &profs_dir, &claude_data)?;
if drift_detected {
    let _ = app.emit("claude-profile-drift-detected", ());
}
```

Cần `use tauri::Emitter;` ở top của file.

Logic phía dưới giữ nguyên — sau reconcile thì meta đã sync với reality.

### Step 3: Sửa `switch_credential_profile`

Thay block:
```rust
let mut meta = config::read_meta(&claude_data);
let current_email = meta.active_profile_name.clone()
    .or_else(|| auth::read_oauth_from_claude_json(&home).and_then(|o| o.email_address))
    .unwrap_or_default();
```

Thành:
```rust
// Reconcile trước — bảo đảm credentials hiện tại được lưu đúng folder nếu có drift
let (current_email_opt, _) = reconcile_active_profile(&home, &claude, &profs_dir, &claude_data)?;
let mut meta = config::read_meta(&claude_data);
let current_email = current_email_opt.unwrap_or_default();
```

Phần backup logic phía dưới giữ nguyên — nhưng giờ `current_email` đảm bảo là email thực tế của credentials hiện tại (vì reconcile đã sync).

Tuy nhiên block backup hiện tại vẫn copy lần nữa — có thể skip nếu drift đã được handle. Đơn giản: giữ nguyên (idempotent — copy đè cùng folder không hại).

### Step 4: Sửa `save_current_as_profile`

Hàm này đã đọc email đúng từ `~/.claude.json`. Chỉ thêm check warning:

```rust
let mut meta = config::read_meta(&claude_data);
let prev_active = meta.active_profile_name.clone();
if prev_active.as_ref() != Some(&email) {
    eprintln!(
        "[save_current] Active profile changed: {:?} -> {}",
        prev_active, email
    );
}
meta.active_profile_name = Some(email.clone());
// ... rest unchanged
```

### Step 5: Validation profile name

Kiểm tra: hiện tại Claude flow KHÔNG gọi `sanitize_profile_name` (chỉ IDE flow gọi). Email là tên folder → cần đảm bảo email không chứa `/`, `\`, `..`, không bắt đầu bằng `.`.

Action: Thêm check tương tự vào `reconcile_active_profile` và `save_current_as_profile` (helper sẵn có trong `ide_manager.rs:8`, có thể move sang `paths.rs` để dùng chung — nhưng để tránh scope creep, inline check tại 2 chỗ này):

```rust
fn validate_email_as_folder(email: &str) -> Result<(), String> {
    if email.is_empty()
        || email.contains('/')
        || email.contains('\\')
        || email.contains("..")
        || email.starts_with('.')
    {
        return Err(format!("Invalid email for folder name: '{}'", email));
    }
    Ok(())
}
```

### Step 6: Compile check

```bash
cd /home/ducnd/My_Project/claude-tools/src-tauri
cargo check
cargo clippy --all-targets -- -D warnings
```

Sửa hết warnings/errors trước khi sang phase 2.

## Todo List

- [ ] Thêm `reconcile_active_profile` helper vào `config_commands.rs`
- [ ] Thêm `validate_email_as_folder` helper
- [ ] Update `list_credential_profiles` gọi reconcile + emit event
- [ ] Update `switch_credential_profile` gọi reconcile thay logic cũ
- [ ] Update `save_current_as_profile` thêm warning log
- [ ] Add `use tauri::Emitter;` import
- [ ] `cargo check` không error
- [ ] `cargo clippy` không warning
- [ ] Manual test: simulate drift bằng cách edit `~/.claude.json` thủ công

## Success Criteria

- Sau fix, simulate scenario: copy 2 credentials A, B; set meta.active_profile_name=A; ghi credentials B vào active_path; gọi `list_credential_profiles` → folder `profiles/A/` còn nguyên, folder `profiles/B/` được tạo, active = B
- `cargo check` pass
- Không break command nào hiện có (rename, delete, get_claude_cli_state)

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| `reconcile_active_profile` fail giữa chừng (e.g., disk full) | Lỗi propagate lên frontend; không corrupt data vì chưa update meta nếu chưa copy xong |
| User có Claude profile dùng custom name (không phải email) | Hiện tại save_current_as_profile đã ép dùng email. Nếu user có folder name khác qua rename → vẫn list được (logic list scan tất cả folder), chỉ là không match active |
| Email rất dài / Unicode | Filesystem hỗ trợ UTF-8 trên cả 3 OS; chỉ chặn ký tự nguy hiểm |

## Security Considerations

- File permission `0o600` được set cho credentials.json sau copy (giữ nguyên hành vi hiện tại)
- Không log content của credentials (chỉ log email)
- Reconcile không expose credentials qua event — chỉ emit event signal `()`

## Next Steps

→ Phase 2: Frontend auto-refresh + drift toast
