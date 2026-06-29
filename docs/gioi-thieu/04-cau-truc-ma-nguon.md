# Cấu Trúc Mã Nguồn

## Tổng quan thư mục

```
agent-switch-tools/
├── src/                          ← Frontend (React + TypeScript)
│   ├── pages/                    ← Các trang chính
│   ├── components/               ← Component giao diện
│   ├── hooks/                    ← Custom React hooks (logic tái sử dụng)
│   ├── lib/                      ← Utilities, types, i18n
│   ├── locales/                  ← File ngôn ngữ (en.json, vi.json)
│   ├── App.tsx                   ← Component gốc
│   └── main.tsx                  ← Entry point
│
├── src-tauri/                    ← Backend (Rust + Tauri)
│   ├── src/
│   │   ├── commands/             ← Các Tauri command (API cho frontend)
│   │   ├── ide/                  ← Module quản lý các IDE
│   │   │   ├── registry.rs       ← IdeType enum + config cho từng IDE
│   │   │   ├── path_helpers.rs   ← Đường dẫn state.vscdb theo OS
│   │   │   ├── sqlite_auth.rs    ← Đọc/ghi auth keys từ SQLite
│   │   │   └── profile_commands.rs ← Tauri commands cho IDE profiles
│   │   ├── tray.rs               ← System tray menu (đa agent)
│   │   ├── quota_refresh_worker.rs ← Background worker
│   │   ├── lib.rs                ← Cấu hình Tauri, đăng ký commands
│   │   └── main.rs               ← Entry point Rust
│   ├── Cargo.toml                ← Dependencies Rust
│   └── tauri.conf.json           ← Cấu hình Tauri (window, permissions...)
│
├── docs/                         ← Tài liệu dự án
├── public/                       ← Assets tĩnh (icons, images)
├── package.json                  ← Dependencies frontend
└── vite.config.ts                ← Cấu hình Vite (build tool)
```

---

## Backend — Rust/Tauri (`src-tauri/src/`)

### Tauri Commands là gì?

Tauri cho phép frontend gọi hàm Rust thông qua **commands**. Giống như API endpoints, nhưng chạy nội bộ trong app thay vì qua HTTP.

```
Frontend (React)                    Backend (Rust)
─────────────────                   ──────────────
invoke("list_profiles")  ────►    fn list_credential_profiles()
     ◄── JSON response ────        → return Vec<Profile>
```

### Danh sách Commands

#### `commands/config_commands.rs` — Quản lý Profile
| Command | Chức năng |
|---|---|
| `list_credential_profiles` | Liệt kê tất cả profile (active + saved) |
| `save_current_as_profile` | Lưu credentials hiện tại thành profile mới |
| `switch_credential_profile` | Chuyển đổi tài khoản |
| `rename_credential_profile` | Đổi tên profile |
| `delete_credential_profile` | Xóa profile |
| `get_claude_cli_state` | Đọc model, session count, .env status |

#### `commands/oauth_commands.rs` — OAuth Account
| Command | Chức năng |
|---|---|
| `get_oauth_account` | Đọc thông tin OAuth từ `~/.claude.json` |
| `save_oauth_for_profile` | Lưu OAuth info vào profile |

#### `commands/quota_commands.rs` — Quota / Usage
| Command | Chức năng |
|---|---|
| `fetch_usage_limits` | Gọi API lấy quota (có cache 2 phút) |
| `fetch_all_profiles_usage` | Lấy quota cho tất cả profile |

#### `commands/token_refresh.rs` — Refresh Token
| Command | Chức năng |
|---|---|
| `refresh_token` | Refresh token cho 1 profile bằng cách chạy Claude CLI |

#### `commands/webhook_commands.rs` — Webhook
| Command | Chức năng |
|---|---|
| `send_webhook` | Gửi usage report tới webhook endpoint |
| `validate_webhook_url` | Kiểm tra URL webhook hợp lệ |

#### `commands/session_usage_commands.rs` — Session Usage
| Command | Chức năng |
|---|---|
| `get_session_usage` | Parse session logs, tính tổng token usage |
| `send_session_usage_webhook` | Gửi session usage report |

#### `commands/device_commands.rs` — Device
| Command | Chức năng |
|---|---|
| `get_device_info` | Đọc device ID, name, hostname |
| `update_device_name` | Cập nhật tên thiết bị |

#### `commands/system_info_commands.rs` — System Info
| Command | Chức năng |
|---|---|
| `get_system_info` | Lấy thông tin OS, CPU, RAM, architecture |

#### `commands/metadata_commands.rs` — Metadata
| Command | Chức năng |
|---|---|
| `get_manager_metadata` | Đọc active profile, lịch sử switch |
| `update_manager_metadata` | Cập nhật metadata |

### Module `modules/` — Tái cấu trúc từ v1.0.11

Từ v1.0.11, toàn bộ logic backend tách thành 4 nhóm rõ ràng thay cho thư mục `ide/` cũ:

```
src-tauri/src/modules/
├── core/                        ← Logic dùng chung giữa các IDE
│   ├── path_helpers.rs          ← Resolve state.vscdb / token file, IDE app dir, profiles dir
│   ├── sqlite_auth.rs           ← Đọc/ghi ItemTable SQLite với retry on-lock
│   ├── credential_source.rs     ← Trừu tượng nguồn creds: Vscdb (SQLite) | JsonFile (CLI) — v1.0.12
│   └── ide_manager.rs           ← Save/read auth-keys.json + resolve email (userinfo)
├── providers/                   ← Triết lý: 1 provider = 1 module
│   ├── mod.rs                   ← Trait IdeProvider + enum IdeType + IdeInfo
│   ├── utils.rs                 ← Helpers dùng chung (ví dụ extract plan từ proto JSON)
│   ├── claude_cli/              ← Claude Code (Anthropic OAuth)
│   │   ├── auth.rs              ← OAuth flow, credentials.json
│   │   ├── config.rs            ← Manager metadata, usage stats
│   │   └── quota.rs             ← Anthropic /api/oauth/usage + stale-cache
│   ├── cursor/                  ← Cursor IDE provider
│   ├── windsurf/                ← Windsurf IDE provider
│   └── antigravity/             ← Antigravity (Google Cloud Code) — 3 biến thể (tạm ẩn từ v1.0.13)
│       ├── mod.rs               ← AntigravityProvider (Desktop) + AntigravityIdeProvider + AntigravityCliProvider
│       ├── oauth.rs             ← Proto parse (theo sentinel key) + OAuth refresh + CLI token + userinfo
│       └── quota.rs             ← retrieveUserQuotaSummary (Weekly + 5h) — v1.0.12 (code vẫn giữ)
├── quota/                       ← Shared types: UsageBucket, UsageLimits
└── shared/                      ← Hạ tầng chung: HTTP client, logger, paths
```

> `build.rs` (thư mục `src-tauri/`) nạp `.env` lúc build để cấp `ANTIGRAVITY_OAUTH_CLIENT_ID/SECRET` cho `option_env!` (CI inject từ secrets).

**Trait `IdeProvider`** chuẩn hóa interface mỗi IDE:
- `auth_keys()` — danh sách key cần đọc/ghi trong nguồn creds (key đầu = key chính, dùng cho `JsonFile`)
- `token_key()` — key chứa access token (None với Antigravity IDE/CLI vì token nằm trong proto/JSON)
- `extract_email()`, `extract_display_name()`, `extract_membership()` — trích thông tin hiển thị
- `normalize_token(raw)` — chuẩn hóa token (Desktop: `apiKey`; CLI: `token.access_token`)

**`CredentialSource`** (v1.0.12) cho phép cùng luồng profile/switch/quota chạy trên 2 loại nguồn:
- `Vscdb(path)` — Cursor/Windsurf/Antigravity Desktop+IDE (SQLite `state.vscdb`)
- `JsonFile(path)` — Antigravity CLI (`~/.gemini/antigravity-cli/antigravity-oauth-token`)

**Provider quota riêng**:
- Claude CLI → Anthropic `/api/oauth/usage`
- Antigravity (cả 3 biến thể) → `retrieveUserQuotaSummary` (Weekly + 5h) + OAuth refresh + userinfo
- Cursor & Windsurf → không có public API; tạm ẩn khỏi dashboard/tray

### Các file đặc biệt

| File | Vai trò |
|---|---|
| `lib.rs` | Đăng ký tất cả commands, khởi tạo Tauri app, start background worker |
| `main.rs` | Entry point — gọi `lib.rs` |
| `tray.rs` | Tạo và quản lý System Tray menu |
| `quota_refresh_worker.rs` | Background task chạy mỗi 5 phút, fetch quota tự động |
| `commands/path_helpers.rs` | Helper functions cho đường dẫn file (home dir, profile dir...) |

---

## Frontend — React/TypeScript (`src/`)

### Pages (Trang)

| File | Mô tả |
|---|---|
| `pages/dashboard.tsx` | **Trang chính** — hiển thị profile active, danh sách saved profiles, nút thêm/switch/xóa |
| `pages/settings-page.tsx` | **Cài đặt** — gồm nhiều tab: General, Webhook, Session Usage, Device, About |

### Components (Thành phần giao diện)

| File | Mô tả |
|---|---|
| `components/profile-card.tsx` | Card hiển thị 1 profile: email, subscription, quota bars, actions |
| `components/profile-form-dialog.tsx` | Dialog thêm/chỉnh sửa profile |
| `components/add-account-dialog.tsx` | Dialog hướng dẫn thêm tài khoản mới |
| `components/switch-confirmation-dialog.tsx` | Dialog xác nhận trước khi switch |
| `components/delete-confirm-dialog.tsx` | Dialog xác nhận xóa profile |
| `components/usage-limits-display.tsx` | Hiển thị thanh progress quota (5h, 7d, Sonnet) |
| `components/cli-status-bar.tsx` | Thanh trạng thái Claude CLI (model, sessions) |
| `components/general-settings-panel.tsx` | Panel cài đặt chung (auto-update, language, theme) |
| `components/webhook-settings-panel.tsx` | Panel cấu hình webhook |
| `components/session-usage-webhook-panel.tsx` | Panel cấu hình session usage webhook |
| `components/device-settings-panel.tsx` | Panel quản lý device name |
| `components/update-notification-dialog.tsx` | Modal thông báo có bản cập nhật mới |
| `components/mode-toggle.tsx` | Nút chuyển Light/Dark mode |
| `components/theme-provider.tsx` | Provider quản lý theme context |

### Hooks (Logic tái sử dụng)

React hooks đóng gói logic phức tạp để components dùng lại.

| File | Mô tả |
|---|---|
| `hooks/use-claude-config.ts` | Đọc cấu hình Claude CLI (model, sessions) |
| `hooks/use-profiles.ts` | CRUD profiles + listen events từ tray |
| `hooks/use-usage-stats.ts` | Fetch + cache quota, listen background updates |
| `hooks/use-device-info.ts` | Quản lý device identity |
| `hooks/use-webhook-config.ts` | Load/save cấu hình webhook |
| `hooks/use-webhook-sender.ts` | Dispatch webhook (startup/change/manual) |
| `hooks/use-app-updater.ts` | Check updates, download, install |
| `hooks/use-auto-update-config.ts` | Bật/tắt auto-update |
| `hooks/use-autostart-config.ts` | Bật/tắt tự khởi động cùng OS |

### Libraries

| File | Mô tả |
|---|---|
| `lib/types.ts` | TypeScript interfaces/types dùng chung |
| `lib/utils.ts` | Utility functions (cn, format...) |
| `lib/i18n.ts` | Cấu hình i18next (đa ngôn ngữ) |
| `lib/settings-store.ts` | Đọc/ghi settings vào Tauri store |

---

## Luồng dữ liệu tổng thể

```
Người dùng
    │
    ▼
┌──────────┐   invoke()   ┌──────────────┐   fs::read   ┌─────────────┐
│ React UI │ ──────────► │ Rust Command │ ──────────► │ ~/.claude/  │
│          │ ◄────────── │              │ ◄────────── │ files       │
│          │   JSON       │              │   data      │             │
└──────────┘              │              │             └─────────────┘
                          │              │
                          │              │   HTTP GET    ┌─────────────┐
                          │              │ ──────────► │ Anthropic   │
                          │              │ ◄────────── │ OAuth API   │
                          └──────────────┘   JSON       └─────────────┘
                                │
                          emit("event")
                                │
                                ▼
                          ┌──────────┐
                          │ React UI │ (cập nhật real-time)
                          └──────────┘
```

---

**Tiếp theo**: [Hướng dẫn sử dụng](05-huong-dan-su-dung.md)
