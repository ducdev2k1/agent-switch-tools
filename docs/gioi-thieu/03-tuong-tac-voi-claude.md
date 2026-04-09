# Tương Tác Với Claude — Giải Thích Kỹ Thuật

Tài liệu này chỉ ra **chính xác** những chỗ trong code mà Claude Tools tương tác với hệ thống Claude Code CLI và Anthropic API.

## Tổng quan các điểm tương tác

```
Claude Tools ←→ Hệ thống Claude
─────────────────────────────────

1. ĐỌC file credentials          ~/.claude/.credentials.json
2. ĐỌC file OAuth                ~/.claude.json
3. ĐỌC settings                  ~/.claude/settings.json
4. ĐỌC lịch sử phiên            ~/.claude/history.jsonl
5. ĐỌC session logs              ~/.claude/projects/**/*.jsonl
6. GHI file credentials           ~/.claude/.credentials.json (khi switch)
7. GHI file OAuth                 ~/.claude.json (khi switch)
8. GỌI Anthropic OAuth API        api.anthropic.com/api/oauth/usage
9. CHẠY Claude CLI                claude -p "hi" (khi refresh token)
```

---

## 1. Đọc Credentials — `config_commands.rs`

### File: `~/.claude/.credentials.json`

Đây là file **quan trọng nhất** — chứa token xác thực để Claude Code hoạt động.

```json
{
  "accessToken": "sk-ant-...",
  "refreshToken": "rt-ant-...",
  "expiresAt": "2026-04-10T12:00:00Z",
  "expiresIn": 3600
}
```

### Code tương tác

**File**: `src-tauri/src/commands/config_commands.rs`

```rust
// Đường dẫn file credentials
fn credentials_path() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(".claude")
        .join(".credentials.json")
}

// Đọc credentials hiện tại
fn read_current_credentials() -> Result<Value> {
    let path = credentials_path();
    let content = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}
```

**Khi nào đọc?**
- Khi app khởi động → hiển thị tài khoản active
- Khi lưu profile mới → backup credentials hiện tại
- Khi kiểm tra token hết hạn

---

## 2. Hoán đổi Credentials — Switch Profile

### Cơ chế swap

**File**: `src-tauri/src/commands/config_commands.rs`

```rust
// Lưu credentials hiện tại vào profile
fn backup_current_credentials(profile_name: &str) -> Result<()> {
    let src = credentials_path();           // ~/.claude/.credentials.json
    let dst = profile_dir(profile_name)     // ~/.claude/.claude-tools/profiles/{name}/
        .join("credentials.json");
    fs::copy(&src, &dst)?;                  // Copy file
    Ok(())
}

// Khôi phục credentials từ profile
fn restore_credentials(profile_name: &str) -> Result<()> {
    let src = profile_dir(profile_name).join("credentials.json");
    let dst = credentials_path();
    fs::copy(&src, &dst)?;                  // Ghi đè file active
    Ok(())
}
```

**Đây là điểm tương tác QUAN TRỌNG NHẤT** — app trực tiếp **ghi đè** file `.credentials.json` của Claude Code. Nhờ vậy, Claude Code sẽ tự động dùng credentials mới mà không cần đăng nhập lại.

---

## 3. Đọc OAuth Account — `oauth_commands.rs`

### File: `~/.claude.json`

Chứa thông tin tài khoản OAuth (không phải token, chỉ là metadata).

```json
{
  "oauthAccount": {
    "emailAddress": "user@gmail.com",
    "accountUuid": "uuid-abc-123",
    "subscriptionType": "claude_pro",
    "organizationName": "My Org",
    "organizationRole": "member"
  }
}
```

**File**: `src-tauri/src/commands/oauth_commands.rs`

```rust
// Đọc thông tin OAuth từ ~/.claude.json
fn read_oauth_account() -> Result<OAuthAccount> {
    let path = dirs::home_dir().unwrap().join(".claude.json");
    let content = fs::read_to_string(&path)?;
    let json: Value = serde_json::from_str(&content)?;
    // Trích xuất field "oauthAccount"
    Ok(parse_oauth_account(&json["oauthAccount"]))
}
```

**Khi nào đọc?**
- Khi hiển thị email, loại subscription trên profile card
- Khi lưu profile mới → backup OAuth info cùng credentials

---

## 4. Gọi Anthropic OAuth API — `quota_commands.rs`

### Đây là kết nối INTERNET DUY NHẤT tới Anthropic

**File**: `src-tauri/src/commands/quota_commands.rs`

```rust
// Gọi API lấy thông tin usage/quota
async fn fetch_usage(access_token: &str) -> Result<UsageResponse> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await?;
    Ok(response.json().await?)
}
```

**Dữ liệu trả về**:

```json
{
  "limits": [
    {
      "type": "5h",
      "utilization": 0.152,    // 15.2% đã dùng
      "resetAt": "2026-04-10T01:00:00Z"
    },
    {
      "type": "7d",
      "utilization": 0.450,
      "resetAt": "2026-04-12T00:00:00Z"
    }
  ]
}
```

**Lưu ý bảo mật**: App CHỈ gửi `accessToken` tới domain `api.anthropic.com` — không gửi tới bất kỳ server nào khác.

---

## 5. Background Quota Worker — `quota_refresh_worker.rs`

### Tự động cập nhật quota mỗi 5 phút

**File**: `src-tauri/src/quota_refresh_worker.rs`

```rust
// Worker chạy ngầm
pub fn start_quota_refresh_worker(app_handle: AppHandle) {
    tokio::spawn(async move {
        loop {
            // Chờ 5 phút
            tokio::time::sleep(Duration::from_secs(300)).await;

            // Fetch quota cho tất cả profiles
            for profile in profiles {
                fetch_and_cache_usage(&profile).await;
                tokio::time::sleep(Duration::from_secs(1)).await; // delay giữa các profile
            }

            // Gửi event tới frontend
            app_handle.emit("usage-updated", &active_usage);
            app_handle.emit("all-profiles-usage-updated", &all_usage);
        }
    });
}
```

**Event flow**:
```
Worker (Rust) ─── emit("usage-updated") ──→ Frontend (React)
                                              │
                                              ▼
                                         Cập nhật UI
                                         (% quota mới)
```

---

## 6. Refresh Token — `token_refresh.rs`

### Khi token hết hạn

**File**: `src-tauri/src/commands/token_refresh.rs`

```rust
// Refresh token bằng cách chạy Claude CLI
async fn refresh_token_for_profile(profile_name: &str) -> Result<()> {
    // 1. Backup credentials hiện tại
    backup_current_credentials("__temp__")?;

    // 2. Swap credentials sang profile cần refresh
    restore_credentials(profile_name)?;

    // 3. Chạy Claude CLI (nó sẽ tự refresh token)
    Command::new("claude")
        .args(["-p", "hi", "--max-turns", "1"])
        .output()
        .await?;

    // 4. Lưu credentials đã refresh
    backup_current_credentials(profile_name)?;

    // 5. Khôi phục credentials gốc
    restore_credentials("__temp__")?;

    Ok(())
}
```

**Tại sao dùng `claude -p "hi"`?**
Claude Code CLI có cơ chế tự động refresh token trước khi chạy. Bằng cách chạy 1 lệnh đơn giản, CLI sẽ refresh token và ghi lại vào file credentials. App chỉ cần copy file đã refresh.

---

## 7. Đọc Session Logs — `session_usage_commands.rs`

### Parse lịch sử phiên làm việc

**File**: `src-tauri/src/commands/session_usage_commands.rs`

```rust
// Đọc session logs từ Claude projects
fn parse_session_logs(period: &str) -> Result<SessionUsage> {
    let projects_dir = dirs::home_dir()
        .unwrap()
        .join(".claude")
        .join("projects");

    // Duyệt tất cả file .jsonl
    for entry in WalkDir::new(&projects_dir) {
        if entry.path().extension() == Some("jsonl") {
            // Parse từng dòng JSON
            for line in BufReader::new(File::open(entry.path())?).lines() {
                let json: Value = serde_json::from_str(&line?)?;
                // Trích xuất token usage
                if let Some(usage) = json["message"]["usage"].as_object() {
                    total_input += usage["input_tokens"].as_u64();
                    total_output += usage["output_tokens"].as_u64();
                }
            }
        }
    }
    Ok(SessionUsage { total_input, total_output, session_count })
}
```

**File được đọc**: `~/.claude/projects/{project-hash}/{session-id}.jsonl`

Mỗi file `.jsonl` chứa log chi tiết của 1 phiên Claude Code, bao gồm: tin nhắn, token usage, tool calls...

---

## 8. Đọc CLI Settings — `config_commands.rs`

### Lấy model đang dùng

**File**: `src-tauri/src/commands/config_commands.rs`

```rust
// Đọc settings của Claude CLI
fn get_claude_cli_state() -> Result<CliState> {
    // Model hiện tại
    let settings_path = home.join(".claude").join("settings.json");
    let model = read_json(&settings_path)?["model"].as_str();

    // Số phiên làm việc
    let history_path = home.join(".claude").join("history.jsonl");
    let session_count = count_lines(&history_path)?;

    Ok(CliState { model, session_count })
}
```

---

## Tóm tắt: Bản đồ tương tác

```
┌──────────────┐                    ┌──────────────────────┐
│ Claude Tools │                    │ Hệ thống Claude      │
│              │                    │                      │
│  Frontend ───┼── invoke ──────►   │                      │
│  (React)     │                    │                      │
│              │                    │                      │
│  Backend ────┼── ĐỌC ────────►   │ .credentials.json    │
│  (Rust)      │                    │ .claude.json         │
│              │                    │ settings.json        │
│              │                    │ history.jsonl        │
│              │                    │ projects/*.jsonl     │
│              │                    │                      │
│              ├── GHI ────────►   │ .credentials.json    │
│              │   (swap only)      │ .claude.json         │
│              │                    │                      │
│              ├── HTTP GET ───►   │ api.anthropic.com    │
│              │   (quota)          │ /api/oauth/usage     │
│              │                    │                      │
│              ├── EXEC ───────►   │ claude -p "hi"       │
│              │   (refresh)        │ (trigger refresh)    │
└──────────────┘                    └──────────────────────┘
```

### Bảo mật tại mỗi điểm tương tác

| Điểm | Rủi ro | Biện pháp |
|---|---|---|
| Đọc credentials | Token lộ ngoài app | Quyền file 0600, không log token |
| Ghi credentials | Ghi nhầm/mất data | Luôn backup trước khi swap |
| Gọi API | Token bị chặn | Chỉ gọi domain Anthropic chính thức |
| Chạy CLI | Quá trình treo | Timeout, non-blocking async |
| Đọc session logs | Chỉ đọc, không sửa | Read-only, không ghi vào .jsonl |

---

**Tiếp theo**: [Cấu trúc mã nguồn](04-cau-truc-ma-nguon.md)
