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

### Module `ide/` — Hỗ trợ đa IDE (mới từ v1.0.10)

| File | Vai trò |
|---|---|
| `ide/registry.rs` | Enum `IdeType` (Cursor/Windsurf/Antigravity) + config (auth keys, email extraction, process names) |
| `ide/path_helpers.rs` | Resolve đường dẫn `state.vscdb` theo OS, check IDE đã cài |
| `ide/sqlite_auth.rs` | Đọc/ghi auth keys từ `ItemTable` trong SQLite, trích email theo từng IDE |
| `ide/profile_commands.rs` | Tauri commands: `list_ide_profiles`, `save_ide_profile`, `switch_ide_profile`, `delete_ide_profile` |

**3 cơ chế trích email** (tùy IDE):
- `DirectKey` — Cursor: email nằm ngay tại key `cursorAuth/cachedEmail`
- `JsonField` — Antigravity: email nằm trong JSON của key `antigravityAuthStatus`
- `ProtoBase64Email` — Windsurf: email encode trong protobuf base64

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
