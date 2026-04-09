# Tổng Quan Dự Án Claude Tools

## Claude Tools là gì?

**Claude Tools** là một ứng dụng desktop (chạy trên máy tính) giúp bạn **quản lý nhiều tài khoản Claude Code CLI** một cách dễ dàng.

### Claude Code CLI là gì?

**Claude Code** là một công cụ dòng lệnh (CLI - Command Line Interface) của công ty Anthropic. Nó cho phép lập trình viên sử dụng trí tuệ nhân tạo Claude ngay trong terminal để viết code, debug, và thực hiện các tác vụ lập trình.

Khi bạn đăng nhập Claude Code, nó lưu thông tin xác thực (credentials) vào một file trên máy tính của bạn. **Vấn đề là**: Claude Code chỉ hỗ trợ **1 tài khoản tại một thời điểm**. Nếu bạn có nhiều tài khoản (ví dụ: tài khoản cá nhân và tài khoản công ty), bạn phải đăng nhập/đăng xuất liên tục — rất bất tiện.

### Claude Tools giải quyết vấn đề gì?

Claude Tools giúp bạn:

1. **Lưu nhiều tài khoản** — Backup credentials của từng tài khoản thành các profile riêng biệt
2. **Chuyển đổi 1-click** — Đổi tài khoản chỉ bằng 1 cú nhấp chuột, không cần đăng nhập lại
3. **Theo dõi hạn mức** — Xem bạn đã dùng bao nhiêu % quota (hạn mức sử dụng) của từng tài khoản
4. **Gửi báo cáo** — Tự động gửi báo cáo sử dụng về webhook (cho team/admin theo dõi)

## Công nghệ sử dụng

| Thành phần | Công nghệ | Vai trò |
|---|---|---|
| **Giao diện** (Frontend) | React 19, TypeScript, Tailwind CSS | Hiển thị UI, tương tác người dùng |
| **Lõi ứng dụng** (Backend) | Rust, Tauri v2 | Đọc/ghi file, gọi API, xử lý logic hệ thống |
| **Thư viện UI** | shadcn/ui, Radix UI | Các component giao diện đẹp, accessible |
| **Đa ngôn ngữ** | i18next | Hỗ trợ Tiếng Anh và Tiếng Việt |
| **Build tool** | Vite 7 (frontend), Cargo (backend) | Biên dịch và đóng gói ứng dụng |

### Tại sao dùng Tauri?

**Tauri** là framework để xây dựng ứng dụng desktop bằng web technology (HTML/CSS/JS) kết hợp với Rust. So với Electron (dùng bởi VS Code, Slack...), Tauri có:

- **Kích thước nhỏ hơn nhiều** (~5-10MB vs ~150MB của Electron)
- **Tốn ít RAM hơn** — dùng WebView của hệ điều hành thay vì bundle cả trình duyệt Chromium
- **Backend Rust** — nhanh, an toàn bộ nhớ, xử lý file/API hiệu quả

## Kiến trúc tổng thể

```
┌─────────────────────────────────────────────────┐
│                  Claude Tools App                │
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
│  ┌──────────┐          ┌─────────────────┐      │
│  │ UI hiển  │          │ Hệ thống file   │      │
│  │ thị cho  │          │ ~/.claude/       │      │
│  │ người    │          │ ~/.claude.json   │      │
│  │ dùng     │          │ Anthropic API    │      │
│  └──────────┘          └─────────────────┘      │
└─────────────────────────────────────────────────┘
```

## Bảo mật

- **100% Offline** — Mọi dữ liệu lưu trên máy cá nhân, không gửi lên server nào
- **Ngoại lệ duy nhất**: Gọi API chính thức của Anthropic để lấy thông tin quota
- **Mã nguồn mở** (MIT License) — Ai cũng có thể kiểm tra code

## Hệ điều hành hỗ trợ

| Hệ điều hành | File cài đặt |
|---|---|
| Windows 10+ | `.msi`, `.exe` |
| macOS Intel | `.dmg` |
| macOS Apple Silicon (M1/M2/M3) | `.dmg` |
| Linux (Ubuntu 22.04+) | `.deb`, `.AppImage` |

---

**Tiếp theo**: [Tính năng chi tiết](02-tinh-nang-chi-tiet.md)
