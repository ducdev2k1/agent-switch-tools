---
sidebar_position: 1
title: Tổng Quan Dự Án
---

## Agent Switch Tools là gì?

**Agent Switch Tools** là một ứng dụng desktop (chạy trên máy tính) giúp bạn **quản lý nhiều tài khoản AI coding agent** (Claude Code, Cursor, Windsurf, Antigravity) từ một nơi duy nhất.

### Các AI coding agent được hỗ trợ

| Agent           | Loại            | Cơ chế lưu credentials                |
| --------------- | --------------- | ------------------------------------- |
| **Claude Code** | CLI (Anthropic) | File `.credentials.json`              |
| **Cursor**      | IDE             | SQLite `state.vscdb`                  |
| **Windsurf**    | IDE             | SQLite `state.vscdb` (protobuf email) |
| **Antigravity** | IDE             | SQLite `state.vscdb` (JSON email)     |

### Vấn đề

Mỗi AI coding agent chỉ hỗ trợ **1 tài khoản tại một thời điểm**. Nếu bạn có nhiều tài khoản (cá nhân / công ty / team), bạn phải đăng nhập/đăng xuất liên tục — rất bất tiện. Chưa kể mỗi agent lưu credentials ở một chỗ khác nhau, khó quản lý.

### Agent Switch Tools giải quyết vấn đề gì?

Agent Switch Tools giúp bạn:

1. **Lưu nhiều tài khoản cho từng agent** — Mỗi agent có kho profile riêng
2. **Chuyển đổi 1-click** — Đổi tài khoản chỉ bằng 1 cú nhấp chuột, không cần đăng nhập lại
3. **Tự động phát hiện IDE** — App tự nhận biết IDE nào đã cài, chỉ hiển thị những cái đang có
4. **Theo dõi hạn mức** — Xem bạn đã dùng bao nhiêu % quota của từng tài khoản Claude Code
5. **Gửi báo cáo** — Tự động gửi báo cáo sử dụng về webhook (cho team/admin theo dõi)

## Công nghệ sử dụng

| Thành phần                 | Công nghệ                          | Vai trò                                     |
| -------------------------- | ---------------------------------- | ------------------------------------------- |
| **Giao diện** (Frontend)   | React 19, TypeScript, Tailwind CSS | Hiển thị UI, tương tác người dùng           |
| **Lõi ứng dụng** (Backend) | Rust, Tauri v2                     | Đọc/ghi file, gọi API, xử lý logic hệ thống |
| **Thư viện UI**            | shadcn/ui, Radix UI                | Các component giao diện đẹp, accessible     |
| **Đa ngôn ngữ**            | i18next                            | Hỗ trợ Tiếng Anh và Tiếng Việt              |
| **Build tool**             | Vite 7 (frontend), Cargo (backend) | Biên dịch và đóng gói ứng dụng              |

### Tại sao dùng Tauri?

**Tauri** là framework để xây dựng ứng dụng desktop bằng web technology (HTML/CSS/JS) kết hợp với Rust. So với Electron (dùng bởi VS Code, Slack...), Tauri có:

- **Kích thước nhỏ hơn nhiều** (~5-10MB vs ~150MB của Electron)
- **Tốn ít RAM hơn** — dùng WebView của hệ điều hành thay vì bundle cả trình duyệt Chromium
- **Backend Rust** — nhanh, an toàn bộ nhớ, xử lý file/API hiệu quả

## Kiến trúc tổng thể

```
┌─────────────────────────────────────────────────┐
│                  Agent Switch Tools App                │
│                                                  │
│  ┌──────────────┐       ┌────────────────────┐  │
│  │   Frontend    │       │     Backend        │  │
│  │   (React)     │◄─────►│     (Rust/Tauri)   │  │
│  │              │ invoke │                    │  │
│  │  Dashboard    │       │  Đọc/ghi files     │  │
│  │  Settings     │       │  Gọi Anthropic API │  │
│  │  System Tray  │       │  Background worker │  │
│  └──────────────┘       └────────────────────┘  │
│                                                  │
│         ▼                        ▼               │
│  ┌──────────┐          ┌──────────────────────┐ │
│  │ UI hiển  │          │ Hệ thống file        │ │
│  │ thị cho  │          │ ~/.claude/           │ │
│  │ người    │          │ ~/.agent-switch-tools/│ │
│  │ dùng     │          │ IDE state.vscdb      │ │
│  │          │          │ Anthropic API        │ │
│  └──────────┘          └──────────────────────┘ │
└─────────────────────────────────────────────────┘
```

## Bảo mật

- **100% Offline** — Mọi dữ liệu lưu trên máy cá nhân, không gửi lên server nào
- **Ngoại lệ duy nhất**: Gọi API chính thức của Anthropic để lấy thông tin quota
- **Mã nguồn mở** (MIT License) — Ai cũng có thể kiểm tra code

## Hệ điều hành hỗ trợ

| Hệ điều hành                   | File cài đặt        |
| ------------------------------ | ------------------- |
| Windows 10+                    | `.msi`, `.exe`      |
| macOS Intel                    | `.dmg`              |
| macOS Apple Silicon (M1/M2/M3) | `.dmg`              |
| Linux (Ubuntu 22.04+)          | `.deb`, `.AppImage` |

---

**Tiếp theo**: [Tính năng chi tiết](tinh-nang.md)
