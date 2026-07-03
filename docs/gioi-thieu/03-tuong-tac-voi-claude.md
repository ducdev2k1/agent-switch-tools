# Tương Tác Với Các Agent — Giải Thích Kỹ Thuật

Tài liệu này chỉ ra **chính xác** những chỗ trong code mà Agent Switch Tools tương tác với từng AI coding agent (Claude Code, Cursor, Windsurf, Antigravity) và Anthropic API.

## Tổng quan các điểm tương tác

### Claude Code (macOS: Keychain + file fallback; Linux/Windows: file-based)

```
macOS:
1. ĐỌC credentials          Keychain (hashed slot) → Keychain (global) → file
2. GHI credentials          Keychain → verify + mirror vào file (nếu tồn tại)

Linux/Windows:
1. ĐỌC credentials          ~/.claude/.credentials.json
2. GHI credentials          ~/.claude/.credentials.json

Chung cả 3 OS:
3. ĐỌC file OAuth                ~/.claude.json
4. ĐỌC settings                  ~/.claude/settings.json
5. ĐỌC lịch sử phiên            ~/.claude/history.jsonl
6. ĐỌC session logs              ~/.claude/projects/**/*.jsonl
7. GHI file OAuth                ~/.claude.json (khi switch)
8. GỌI Anthropic OAuth API       api.anthropic.com/api/oauth/usage
9. CHẠY Claude CLI               claude -p "hi" (khi refresh token)
```

### IDE (SQLite-based)

```
1. ĐỌC state.vscdb (SQLite)      Auth keys từ ItemTable
2. TRÍCH email                   DirectKey / JsonField / ProtoBase64Email
3. GHI state.vscdb               UPDATE ItemTable khi switch
4. PHÁT HIỆN IDE                 Check state.vscdb tồn tại

Đường dẫn state.vscdb theo OS:
  Linux:    ~/.config/{AppName}/User/globalStorage/state.vscdb
  macOS:    ~/Library/Application Support/{AppName}/User/globalStorage/state.vscdb
  Windows:  %APPDATA%/{AppName}/User/globalStorage/state.vscdb

AppName: Cursor | Windsurf | Antigravity
```

---

## 1. Đọc Credentials — `config_commands.rs` & `active_store.rs`

### Cơ chế lưu trữ

Credentials được lưu ở các vị trí khác nhau tùy theo hệ điều hành:

**macOS (Claude Code 2.x)**:
- **Slot chính**: `Keychain` → service name = `Claude Code-credentials-<sha256(config_dir)[:8]>`
- **Fallback slot**: `Keychain` → service name = `Claude Code-credentials` (CLI phiên bản cũ)
- **File fallback**: `~/.claude/.credentials.json` (chỉ mirror khi file đã tồn tại, không tự tạo)

**Linux/Windows**:
- **Slot chính**: `~/.claude/.credentials.json`

### Nội dung credentials

```json
{
  "accessToken": "sk-ant-...",
  "refreshToken": "rt-ant-...",
  "expiresAt": "2026-04-10T12:00:00Z",
  "expiresIn": 3600
}
```

### Code tương tác

**File**: `src-tauri/src/modules/shared/active_store.rs`

```rust
// Đọc credentials từ bất kỳ backend nào (Keychain → global → file)
pub fn read_active(&self) -> Option<String> {
    if Self::use_keychain() {
        // macOS: thử hashed slot trước
        if let Some(blob) = read_keychain_blob(&self.hashed_service()) {
            return Some(blob);
        }
        // Fallback tới global slot (CLI cũ)
        if let Some(blob) = read_keychain_blob(GLOBAL_SERVICE) {
            return Some(blob);
        }
    }
    // Linux/Windows: file. macOS: file fallback nếu Keychain không có
    read_file_blob(&self.creds_file)
}
```

**Hashed service name** (macOS):
- Format: `Claude Code-credentials-<sha256(config_dir)[:8]>` (8 ký tự hex đầu tiên)
- Ví dụ: `Claude Code-credentials-a1b2c3d4`
- Bảo đảm mỗi config dir (`~/.claude`) có slot Keychain riêng

**Khi nào đọc?**
- Khi app khởi động → hiển thị tài khoản active
- Khi lưu profile mới → backup credentials hiện tại
- Khi kiểm tra token hết hạn

---

## 2. Hoán đổi Credentials — Switch Profile

### Cơ chế swap

**Saved profiles** (lưu trong `~/.agent-switch-tools/claude/profiles/{email}/credentials.json`) luôn là **file thường** (không thay đổi).

**Active credential** (đang dùng bây giờ) được ghi vào:
- **macOS**: Keychain + mirror vào file (nếu file đã tồn tại)
- **Linux/Windows**: File

**File**: `src-tauri/src/modules/shared/active_store.rs`

```rust
// Ghi credentials vào active backend (Keychain trên macOS, file trên Linux/Windows)
pub fn write_active(&self, blob: &str) -> Result<(), String> {
    if Self::use_keychain() {
        // macOS: chọn service slot (hashed → global → mặc định hashed)
        let service = if keychain_service_exists(&self.hashed_service()) {
            self.hashed_service()
        } else if keychain_service_exists(GLOBAL_SERVICE) {
            GLOBAL_SERVICE.to_string()
        } else {
            self.hashed_service()
        };
        
        // Ghi vào Keychain + confirm + retry nếu cần
        let account = read_keychain_account(&service).unwrap_or_else(current_user);
        let wrote = write_keychain_blob(&service, &account, blob);
        
        // Mirror vào file nếu file đã tồn tại (cho khi Keychain bị lock)
        if self.creds_file.exists() {
            let _ = write_file_blob(&self.creds_file, blob);
        }
        
        if wrote {
            return Ok(());
        }
        // Fallback: nếu Keychain fail, ghi vào file nếu tồn tại
        if self.creds_file.exists() {
            return write_file_blob(&self.creds_file, blob);
        }
        return Err("Failed to write credential to macOS Keychain".to_string());
    }
    // Linux/Windows: ghi file
    write_file_blob(&self.creds_file, blob)
}

// Lưu credentials hiện tại vào saved profile (luôn là file)
fn backup_current_credentials(profile_name: &str) -> Result<()> {
    let active_blob = active_store.read_active()?;    // Đọc từ Keychain/file
    let profile_path = profile_dir(profile_name).join("credentials.json");
    write_file_blob(&profile_path, &active_blob)?;    // Ghi vào profile
    Ok(())
}

// Khôi phục credentials từ saved profile vào active
fn restore_credentials(profile_name: &str) -> Result<()> {
    let profile_path = profile_dir(profile_name).join("credentials.json");
    let blob = read_file_blob(&profile_path)?;        // Đọc từ profile file
    active_store.write_active(&blob)?;                // Ghi vào active (Keychain/file)
    Ok(())
}
```

**Ghi vào Keychain (macOS)** — quy trình:
1. Chạy `security add-generic-password -U -A -s <service> -a <account> -w <blob>`
   - `-U`: upsert (update nếu tồn tại)
   - `-A`: allow any app to read (không cần prompt)
   - Blob được truyền qua `-w` (briefly visible trong process list)
2. **Confirm write**: đọc lại từ Keychain để verify dữ liệu đúng
3. **Retry 3 lần** nếu fail (handle transient lock issues)
4. **Mirror tới file**: nếu file đã tồn tại (keep a readable copy khi Keychain locked)

**Đây là điểm tương tác QUAN TRỌNG NHẤT** — app ghi credentials vào Keychain (macOS) hoặc file (Linux/Windows). Nhờ vậy, Claude Code sẽ tự động dùng credentials mới mà không cần đăng nhập lại.

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

Quy trình refresh token hoạt động với cả **Keychain (macOS)** và **file (Linux/Windows)**:

```rust
// Refresh token bằng cách chạy Claude CLI
async fn refresh_token_for_profile(profile_name: &str) -> Result<()> {
    // 1. Backup active credentials (đọc từ Keychain hoặc file)
    backup_current_credentials("__temp__")?;

    // 2. Swap credentials sang profile cần refresh (ghi vào Keychain/file)
    restore_credentials(profile_name)?;

    // 3. Chạy Claude CLI (nó sẽ tự refresh token từ Keychain/file)
    Command::new("claude")
        .args(["-p", "hi", "--max-turns", "1"])
        .output()
        .await?;

    // 4. Lưu credentials đã refresh (đọc lại từ Keychain/file)
    backup_current_credentials(profile_name)?;

    // 5. Khôi phục credentials gốc (ghi lại vào Keychain/file)
    restore_credentials("__temp__")?;

    Ok(())
}
```

**Flow chi tiết**:
```
App ──┐
      ├─► (backup active) ──► đọc Keychain/file ──► lưu vào profile
      │
      ├─► (restore profile) ──► ghi vào Keychain/file (với confirm + retry)
      │
      ├─► (run claude CLI) ──► Claude đọc từ Keychain/file
      │                         └─► refresh token
      │                         └─► ghi lại vào Keychain/file
      │
      ├─► (backup profile) ──► đọc Keychain/file ──► update profile
      │
      └─► (restore original) ──► ghi lại vào Keychain/file
```

**Tại sao dùng `claude -p "hi"`?**
Claude Code CLI có cơ chế tự động refresh token trước khi chạy. Bằng cách chạy 1 lệnh đơn giản, CLI sẽ:
1. Đọc credentials từ Keychain (macOS) hoặc file (Linux/Windows)
2. Refresh token nếu hết hạn
3. Ghi lại vào Keychain/file

App chỉ cần đọc lại credentials đã refresh từ Keychain/file.

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

## 9. Tương tác với IDE (Cursor / Windsurf / Antigravity)

### Cấu hình IDE registry — `ide/registry.rs`

Mỗi IDE có cấu hình riêng:

```rust
IdeType::Cursor => IdeConfig {
    display_name: "Cursor",
    app_dir_name: "Cursor",
    auth_keys: &[
        "cursorAuth/accessToken",
        "cursorAuth/refreshToken",
        "cursorAuth/cachedEmail",
        ...
    ],
    email_key: EmailKeySource::DirectKey("cursorAuth/cachedEmail"),
    ...
}
```

**3 cách trích email tùy theo IDE**:
- `DirectKey("cursorAuth/cachedEmail")` — Cursor lưu email trực tiếp
- `JsonField("antigravityAuthStatus", "email")` — Antigravity lưu email trong JSON
- `ProtoBase64Email("windsurfAuthStatus", "userStatusProtoBinaryBase64")` — Windsurf encode email trong protobuf base64

### Đọc/ghi SQLite state.vscdb — `ide/sqlite_auth.rs`

```rust
// Đọc auth keys từ ItemTable
pub fn read_ide_auth_keys(db_path: &Path, keys: &[&str]) -> Result<HashMap<String, String>> {
    let conn = Connection::open(db_path)?;
    let mut result = HashMap::new();
    for key in keys {
        let value: String = conn.query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            [key],
            |row| row.get(0),
        )?;
        result.insert(key.to_string(), value);
    }
    Ok(result)
}

// Ghi auth keys khi switch (UPDATE or INSERT)
pub fn write_ide_auth_keys(db_path: &Path, auth_data: &HashMap<String, String>) -> Result<()> {
    let conn = Connection::open(db_path)?;
    for (key, value) in auth_data {
        conn.execute(
            "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
    }
    Ok(())
}
```

### Phát hiện IDE đã cài — `ide/path_helpers.rs`

```rust
// IDE coi như "đã cài" nếu file state.vscdb tồn tại
pub fn ide_is_installed(app: &AppHandle, ide_type: &IdeType) -> bool {
    ide_db_path(app, ide_type)
        .map(|p| p.exists())
        .unwrap_or(false)
}
```

---

## Tóm tắt: Bản đồ tương tác

**macOS**:
```
┌────────────────────┐         ┌──────────────────────┐
│ Agent Switch Tools │         │ macOS Keychain       │
│                    │         │                      │
│  Backend (Rust)    │ GHI ───►│ Claude Code-         │
│                    │ ĐỌC ◄───│ credentials-<hash>   │
│                    │         │ (or -credentials)    │
└────────────────────┘         └──────────────────────┘
                                        │
                                mirror (nếu file tồn tại)
                                        │
                                        ▼
                               ~/.claude/.credentials.json
```

**Linux / Windows**:
```
┌────────────────────┐         ┌──────────────────────┐
│ Agent Switch Tools │ GHI     │ ~/.claude/           │
│                    │ ──────►│ .credentials.json    │
│  Backend (Rust)    │ ĐỌC     │                      │
│                    │ ◄──────│                      │
└────────────────────┘         └──────────────────────┘
```

**Claude Code (chung cả 3 OS)**:
```
┌──────────────────────┐
│ Claude Code          │
│                      │
│ ĐỌC ───────────────►│ .claude.json (OAuth info)
│ ĐỌC ───────────────►│ settings.json
│ ĐỌC ───────────────►│ history.jsonl
│ ĐỌC ───────────────►│ projects/**/*.jsonl
│ GHI ───────────────►│ .claude.json
│                      │
│ HTTP GET ──────────►│ api.anthropic.com (quota)
│ EXEC ──────────────►│ claude -p "hi" (refresh token)
└──────────────────────┘
```

**IDE (Cursor / Windsurf / Antigravity)**:
```
┌────────────────────┐         ┌──────────────────────┐
│ Agent Switch Tools │         │ IDE state.vscdb      │
│                    │         │ (SQLite ItemTable)   │
│  Backend (Rust)    │ GHI ───►│ auth keys            │
│                    │ ĐỌC ◄───│                      │
└────────────────────┘         └──────────────────────┘
```

### Bảo mật tại mỗi điểm tương tác

| Điểm | Rủi ro | Biện pháp |
|---|---|---|
| **Đọc credentials** | Token lộ ngoài app | macOS: Keychain (encrypted by OS) + file fallback (0600). Linux/Windows: file 0600 |
| **Ghi credentials** | Ghi nhầm/mất data | macOS: Keychain + confirm verify + retry 3x. Luôn backup trước khi swap |
| **Keychain write** (macOS) | Transient lock | Retry 3 lần, 200ms delay giữa các lần |
| **Keychain mirror** (macOS) | Token plaintext tạm | Chỉ mirror nếu file đã tồn tại (không tự tạo). File được set 0600 |
| **Gọi API** | Token bị chặn | Chỉ gọi domain Anthropic chính thức (api.anthropic.com) |
| **Chạy CLI** | Quá trình treo | Timeout, non-blocking async |
| **Đọc session logs** | Chỉ đọc, không sửa | Read-only, không ghi vào .jsonl |

---

**Tiếp theo**: [Cấu trúc mã nguồn](04-cau-truc-ma-nguon.md)
