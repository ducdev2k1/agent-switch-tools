# Phase 3 — Tests: Regression coverage

**Status:** Pending
**Priority:** Medium
**Effort:** Small
**Depends on:** Phase 1, Phase 2

## Context Links

- Bug scenario: [plan.md](./plan.md#problem-statement)
- Backend logic: [phase-01-backend-reconcile-active-profile.md](./phase-01-backend-reconcile-active-profile.md)

## Overview

Viết test để đảm bảo bug không regress. Vì code dùng `tauri::AppHandle` (khó mock), tập trung vào **integration tests cho helper functions** + **manual test checklist**.

## Requirements

### Functional
- Test `reconcile_active_profile` với 4 scenarios:
  1. No credentials → return `(None, false)`
  2. credentials + meta sync → return `(Some(email), false)`, không touch FS
  3. credentials + meta drift → save to new folder, update meta, return `(Some(email), true)`
  4. credentials nhưng oauth không có email → return `(None, false)`
- Test `validate_email_as_folder` với edge cases: `..`, `/`, `\`, empty, `.hidden`

### Non-functional
- Tests không phụ thuộc HOME thực (dùng `tempfile::TempDir`)
- Chạy được offline (no network)

## Architecture

```
tests/
  reconcile_active_profile_tests.rs  (new — integration tests)

scenarios:
  setup_temp_claude_dir() ──► create fake ~/.claude/, ~/.claude.json
                          ──► write fake .credentials.json
                          ──► call reconcile_active_profile()
                          ──► assert FS state + return value
```

## Related Code Files

**Modify:**
- `src-tauri/Cargo.toml` — add `tempfile` to `[dev-dependencies]` nếu chưa có

**Create:**
- `src-tauri/src/commands/config_commands.rs` — refactor `reconcile_active_profile` thành `pub(crate)` để test
- `src-tauri/tests/reconcile_active_profile_tests.rs` — integration tests

**Read for context:**
- `src-tauri/Cargo.toml` — kiểm tra dev-dependencies hiện có

## Implementation Steps

### Step 1: Refactor để testable

`reconcile_active_profile` hiện take `&PathBuf` cho từng path → đã testable. Đổi visibility thành `pub(crate)` và move ra module riêng nếu cần:

Option A (đơn giản): expose hàm trong `config_commands.rs` qua `pub(crate)`, test gọi trực tiếp.

Option B (sạch hơn): Move helper sang `modules/providers/claude_cli/reconcile.rs`, import từ `config_commands.rs`.

→ Chọn **Option B** để tách side-effect logic khỏi tauri command boundary.

```rust
// src-tauri/src/modules/providers/claude_cli/reconcile.rs
use std::path::PathBuf;
use crate::modules::providers::claude_cli::{auth, config};

pub fn reconcile_active_profile(
    home: &PathBuf,
    claude: &PathBuf,
    profs_dir: &PathBuf,
    claude_data: &PathBuf,
) -> Result<(Option<String>, bool), String> {
    // ... (logic từ phase 1)
}
```

Update `mod.rs`: `pub mod reconcile;`

### Step 2: Add tempfile dependency

Kiểm tra `Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3"
```

### Step 3: Viết tests

`src-tauri/tests/reconcile_active_profile_tests.rs`:

```rust
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use claude_tools_lib::modules::providers::claude_cli::reconcile::reconcile_active_profile;

struct TestEnv {
    _tmp: TempDir,
    home: PathBuf,
    claude: PathBuf,
    profs_dir: PathBuf,
    claude_data: PathBuf,
}

fn setup() -> TestEnv {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let claude = home.join(".claude");
    let claude_data = home.join(".agent-switch-tools").join("claude");
    let profs_dir = claude_data.join("profiles");
    fs::create_dir_all(&claude).unwrap();
    fs::create_dir_all(&profs_dir).unwrap();
    TestEnv { _tmp: tmp, home, claude, profs_dir, claude_data }
}

fn write_claude_json(home: &PathBuf, email: &str) {
    let json = serde_json::json!({
        "claudeAiOauth": {
            "emailAddress": email,
            "loginType": "oauth",
            "expiresAt": 9999999999000_i64,
        }
    });
    fs::write(home.join(".claude.json"), json.to_string()).unwrap();
}

fn write_active_credentials(claude: &PathBuf, content: &str) {
    fs::write(claude.join(".credentials.json"), content).unwrap();
}

fn write_meta(claude_data: &PathBuf, active_name: Option<&str>) {
    fs::create_dir_all(claude_data).unwrap();
    let meta = serde_json::json!({
        "activeProfileName": active_name,
    });
    fs::write(claude_data.join("meta.json"), meta.to_string()).unwrap();
}

#[test]
fn no_credentials_returns_none() {
    let env = setup();
    let result = reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (None, false));
}

#[test]
fn synced_meta_no_drift() {
    let env = setup();
    write_claude_json(&env.home, "alice@example.com");
    write_active_credentials(&env.claude, r#"{"claudeAiOauth":{"accessToken":"a"}}"#);
    write_meta(&env.claude_data, Some("alice@example.com"));

    let result = reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (Some("alice@example.com".into()), false));
    // Folder không tự động tạo khi không drift
    assert!(!env.profs_dir.join("alice@example.com").exists());
}

#[test]
fn drift_detected_saves_to_new_folder() {
    let env = setup();
    // Setup: meta nói "alice" nhưng credentials thực là "bob"
    write_claude_json(&env.home, "bob@example.com");
    let new_creds = r#"{"claudeAiOauth":{"accessToken":"bob_token"}}"#;
    write_active_credentials(&env.claude, new_creds);
    write_meta(&env.claude_data, Some("alice@example.com"));

    // Tạo sẵn folder alice với credentials cũ
    let alice_dir = env.profs_dir.join("alice@example.com");
    fs::create_dir_all(&alice_dir).unwrap();
    fs::write(alice_dir.join("credentials.json"), r#"{"claudeAiOauth":{"accessToken":"alice_token"}}"#).unwrap();

    let result = reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (Some("bob@example.com".into()), true));

    // CRITICAL: alice folder phải còn nguyên, KHÔNG bị đè
    let alice_creds = fs::read_to_string(alice_dir.join("credentials.json")).unwrap();
    assert!(alice_creds.contains("alice_token"), "Alice credentials phải còn nguyên");

    // bob folder phải được tạo với credentials mới
    let bob_dir = env.profs_dir.join("bob@example.com");
    assert!(bob_dir.exists());
    let bob_creds = fs::read_to_string(bob_dir.join("credentials.json")).unwrap();
    assert!(bob_creds.contains("bob_token"));

    // meta phải update sang bob
    let meta_content = fs::read_to_string(env.claude_data.join("meta.json")).unwrap();
    assert!(meta_content.contains("bob@example.com"));
}

#[test]
fn missing_email_returns_none() {
    let env = setup();
    write_active_credentials(&env.claude, "{}");
    fs::write(env.home.join(".claude.json"), r#"{"claudeAiOauth":{}}"#).unwrap();

    let result = reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (None, false));
}
```

### Step 4: Validate email folder name tests

Inline trong cùng file hoặc trong `config_commands.rs` `#[cfg(test)]` block:

```rust
#[cfg(test)]
mod email_validation_tests {
    use super::validate_email_as_folder;

    #[test]
    fn rejects_empty() {
        assert!(validate_email_as_folder("").is_err());
    }
    #[test]
    fn rejects_path_traversal() {
        assert!(validate_email_as_folder("../etc/passwd").is_err());
    }
    #[test]
    fn rejects_slashes() {
        assert!(validate_email_as_folder("a/b@x.com").is_err());
        assert!(validate_email_as_folder("a\\b@x.com").is_err());
    }
    #[test]
    fn rejects_hidden() {
        assert!(validate_email_as_folder(".hidden@x.com").is_err());
    }
    #[test]
    fn accepts_valid_email() {
        assert!(validate_email_as_folder("alice@example.com").is_ok());
        assert!(validate_email_as_folder("alice+tag@example.co.uk").is_ok());
    }
}
```

### Step 5: Run tests

```bash
cd /home/ducnd/My_Project/claude-tools/src-tauri
cargo test --test reconcile_active_profile_tests
cargo test --lib email_validation_tests
```

### Step 6: Manual test checklist (in addition to unit tests)

Document trong `tests/MANUAL_TEST_CHECKLIST.md` (optional, hoặc inline plan):

- [ ] Login A → Save → Login B (ngoài app) → Open app → Verify A và B đều có
- [ ] Switch B→A → Verify A active, B saved
- [ ] Switch A→B → Verify B active, A saved (existing folder)
- [ ] Delete profile A → Verify chỉ A bị xoá
- [ ] Rename profile → Verify works as before
- [ ] Tray menu refresh đúng sau drift detection

## Todo List

- [ ] Refactor `reconcile_active_profile` ra module riêng (`reconcile.rs`)
- [ ] Add `tempfile` to dev-dependencies nếu chưa có
- [ ] Viết 4 integration tests cho `reconcile_active_profile`
- [ ] Viết 5 unit tests cho `validate_email_as_folder`
- [ ] `cargo test` all pass
- [ ] Run manual test checklist
- [ ] Commit với message `test: add regression tests for claude login drift fix`

## Success Criteria

- Tất cả tests pass
- `cargo test` không có warning
- Manual checklist 6/6 pass

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Test phụ thuộc filesystem timing (atomic ops) | Dùng `tempfile::TempDir` — auto cleanup, isolated per test |
| Library crate name không match `claude_tools_lib` | Kiểm tra `[lib]` trong `Cargo.toml`, adjust import path |
| `serde_json::json!` macro không serialize đúng `claudeAiOauth` shape | Tham khảo struct `OAuthAccount` trong `auth.rs` để khớp field names (camelCase) |

## Security Considerations

- Tests không leak credentials thật (chỉ dùng fake tokens "alice_token", "bob_token")
- TempDir tự cleanup → no leftover files
- File permission test (0o600) có thể skip vì là OS-specific behavior

## Next Steps

- Sau khi tests pass → squash commit, push branch
- Tạo PR với link tới plan này
