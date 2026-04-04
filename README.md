<div align="center">
  <img src="public/apple-touch-icon.png" alt="Claude Tools Logo" width="120" />
  <h1>Claude Tools</h1>
  <p><b>A powerful, lightning-fast, and 100% offline desktop application</b> to manage multiple Claude Code CLI accounts — effortlessly switch profiles, monitor quota usage, and stay meticulously organized.</p>
</div>

![Tauri](https://img.shields.io/badge/Tauri-v2-blue)
![React](https://img.shields.io/badge/React-19-61DAFB)
![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6)
![License](https://img.shields.io/badge/License-MIT-green)

*(Looking for the Vietnamese version? Scroll down / Kéo xuống dưới để xem bản Tiếng Việt)*

## 🌟 Features

- **Multi-Account Management:** Seamlessly add, edit, securely backup, and delete Claude Code CLI credential profiles.
- **Smart Quota Refresh:** Automatically fetches and updates your token usage limits in the background (via Anthropic OAuth API) with intelligent rate-limiting protection.
- **One-Click Switching:** Effortlessly switch between active accounts with an elegant confirmation dialog. No more manual file renaming.
- **System Tray Integration:** Access quick-switching directly from your OS menu bar/system tray without opening the main window.
- **Built-in Auto Updater:** Automatically checks for and installs new releases via GitHub.
- **i18n & Theming:** English and Vietnamese language support with native Light/Dark mode toggles.

## 🔒 Security & Privacy (100% Local)

Because this application manages your `.credentials.json` files (which contain highly sensitive Claude API tokens), **Security is our top priority**:
- **Zero Telemetry & No Tracking**: We guarantee that the app **never** sends your credentials, tokens, or personal data to any external server (except official Anthropic endpoints for quota fetching).
- **100% Offline/Local Storage**: All file parsing, account switching, and management happen purely on your local filesystem using secure Rust bindings. Your API keys never leave your machine.
- **Open Source Transparency**: The codebase is completely open source, allowing you to audit and verify every single system call. You can trust this tool to protect your data.

## 🛠️ Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 19, TypeScript, Vite 7, Tailwind CSS 4 |
| Backend | Rust, Tauri v2 |
| UI | Radix UI, shadcn/ui, Lucide Icons |
| Package Manager | pnpm |

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 10+
- [Rust](https://rustup.rs/) (stable)
- Linux: system dependencies (see below)

### Linux System Dependencies

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev
```

### Install & Run (Development)

```bash
pnpm install
pnpm tauri:dev
```

### Build for Production

```bash
pnpm tauri build
```

## ⚠️ Windows Installation Note

Windows SmartScreen may show a warning when installing new releases of unsigned open-source apps. This is expected behavior — **not a virus**.

**To install:**
1. Click **"More info"** on the SmartScreen popup
2. Click **"Run anyway"**

Alternatively, you can download the `.msi` installer instead of the `.exe` — MSI installers are generally better trusted by Windows.

> We are working on integrating code signing to eliminate this warning in future releases.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
---

<div align="center">
  <h2>🇻🇳 Phiên bản Tiếng Việt</h2>
</div>

## 🌟 Tính năng nổi bật

- **Quản lý Đa tài khoản:** Dễ dàng thêm, chỉnh sửa, sao lưu an toàn và xóa các profile cấu hình của Claude Code CLI.
- **Làm mới Hạn mức Thông minh:** Ứng dụng tự động chạy ngầm và gọi API (Anthropic OAuth API) để cập nhật phần trăm hạn mức sử dụng (Quota) của tài khoản, tích hợp cơ chế chống spam (Throttling) an toàn.
- **Chuyển đổi 1-Click:** Đổi tài khoản làm việc trong tích tắc kèm hộp thoại xác nhận. Không còn cảnh phải đổi tên file thủ công.
- **Tích hợp Khay hệ thống (System Tray):** Chuyển đổi tài khoản nhanh chóng chỉ bằng cách nhấp chuột phải vào biểu tượng ứng dụng ở góc nhỏ màn hình.
- **Tự động Cập nhật:** Ứng dụng tự động kiểm tra và cài đặt phiên bản mới nhất từ Github Releases.
- **Đa cấu hình:** Hỗ trợ song ngữ (Anh/Việt) và Chế độ Sáng/Tối mượt mà.

## 🔒 Bảo mật & Riêng tư (Local 100%)

Bởi vì ứng dụng này trực tiếp quản lý các file `.credentials.json` (chứa Private Token rất nhạy cảm của bạn), **Bảo mật là tiêu chí số 1**:
- **Không theo dõi & Đo lường**: Chúng tôi đảm bảo rằng ứng dụng **không bao giờ** lén thu thập hay gửi token/dữ liệu cá nhân sang bất kì máy chủ bên thứ ba nào (Ngoại trừ việc gọi qua API chính chủ của Anthropic để lấy số liệu Quota).
- **Lưu trữ Offline 100%**: Mọi thao tác xử lý, đọc định dạng, chuyển đổi file đều được thao tác trực tiếp ở cấp độ lõi Rust trên ổ cứng cá nhân của bạn. Token không bao giờ rời khỏi máy của bạn.
- **Mã nguồn mở minh bạch**: Toàn bộ mã nguồn được công khai. Ai cũng có thể kiểm định và soi từng dòng System Call. Bạn hoàn toàn có thể yên tâm.

## 🛠️ Công nghệ sử dụng

| Lớp | Công nghệ |
|-------|------------|
| Frontend | React 19, TypeScript, Vite 7, Tailwind CSS 4 |
| Backend | Rust, Tauri v2 |
| Giao diện | Radix UI, shadcn/ui, Lucide Icons |
| Quản lý Package | pnpm |

## 🚀 Hướng dẫn Cài đặt & Khởi chạy

### Yêu cầu hệ thống

- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 10+
- [Rust](https://rustup.rs/) (phiên bản ổn định)
- Đối với Linux: Phải cài thêm thư viện hệ thống (Xem bên dưới)

### Thư viện hệ thống cho Linux

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev
```

### Chạy Môi trường Phát triển (Dev)

```bash
pnpm install
pnpm tauri:dev
```

### Build ra File cài đặt (.exe, .dmg, .AppImage)

```bash
pnpm tauri build
```

## ⚠️ Lưu ý khi Cài đặt trên Windows

Windows SmartScreen có thể hiện cảnh báo khi cài đặt bản mới của ứng dụng mã nguồn mở chưa ký số. Đây là hành vi bình thường — **không phải virus**.

**Để cài đặt:**
1. Nhấn **"More info"** trên popup SmartScreen
2. Nhấn **"Run anyway"**

Hoặc bạn có thể tải file `.msi` thay vì `.exe` — file MSI thường được Windows tin tưởng hơn.

> Chúng tôi đang triển khai code signing để loại bỏ cảnh báo này trong các bản phát hành tương lai.

## 📝 Giấy phép

Dự án này được phân phối dưới sự cấp phép của Giấy phép MIT - xem file [LICENSE](LICENSE) để biết thêm chi tiết.
