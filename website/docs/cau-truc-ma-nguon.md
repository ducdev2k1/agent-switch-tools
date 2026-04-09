---
sidebar_position: 4
title: Cấu Trúc Mã Nguồn
---

# Cấu Trúc Mã Nguồn

## Tổng quan thư mục

```
claude-tools/
├── src/                          ← Frontend (React + TypeScript)
│   ├── pages/                    ← Các trang chính
│   ├── components/               ← Component giao diện
│   ├── hooks/                    ← Custom React hooks
│   ├── lib/                      ← Utilities, types, i18n
│   ├── locales/                  ← File ngôn ngữ (en.json, vi.json)
│   ├── App.tsx                   ← Component gốc
│   └── main.tsx                  ← Entry point
│
├── src-tauri/                    ← Backend (Rust + Tauri)
│   ├── src/
│   │   ├── commands/             ← Các Tauri command (API cho frontend)
│   │   ├── tray.rs               ← System tray menu
│   │   ├── quota_refresh_worker.rs ← Background worker
│   │   ├── lib.rs                ← Cấu hình Tauri
│   │   └── main.rs               ← Entry point Rust
│   ├── Cargo.toml                ← Dependencies Rust
│   └── tauri.conf.json           ← Cấu hình Tauri
│
├── docs/                         ← Tài liệu dự án
├── public/                       ← Assets tĩnh
├── package.json                  ← Dependencies frontend
└── vite.config.ts                ← Cấu hình Vite
```

---

## Backend Commands

### `config_commands.rs` — Quản lý Profile
| Command | Chức năng |
|---|---|
| `list_credential_profiles` | Liệt kê tất cả profile |
| `save_current_as_profile` | Lưu credentials hiện tại thành profile mới |
| `switch_credential_profile` | Chuyển đổi tài khoản |
| `rename_credential_profile` | Đổi tên profile |
| `delete_credential_profile` | Xóa profile |
| `get_claude_cli_state` | Đọc model, session count |

### `quota_commands.rs` — Quota / Usage
| Command | Chức năng |
|---|---|
| `fetch_usage_limits` | Gọi API lấy quota (cache 2 phút) |
| `fetch_all_profiles_usage` | Lấy quota cho tất cả profile |

### `token_refresh.rs` — Refresh Token
| Command | Chức năng |
|---|---|
| `refresh_active_token` | Refresh token tài khoản active |
| `refresh_profile_token` | Refresh token bất kỳ profile nào |

### `webhook_commands.rs` — Webhook
| Command | Chức năng |
|---|---|
| `send_webhook` | Gửi usage report tới webhook |
| `validate_webhook_url` | Kiểm tra URL hợp lệ |

---

## Frontend Components

| File | Mô tả |
|---|---|
| `pages/dashboard.tsx` | Trang chính — profile list, switch, quota |
| `pages/settings-page.tsx` | Cài đặt — General, Webhook, Device, About |
| `components/profile-card.tsx` | Card profile: email, quota bars, actions |
| `components/usage-limits-display.tsx` | Thanh progress quota |
| `components/cli-status-bar.tsx` | Trạng thái Claude CLI |

---

## Luồng dữ liệu

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
```
