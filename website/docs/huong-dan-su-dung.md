---
sidebar_position: 5
title: Hướng Dẫn Sử Dụng
---

## Yêu cầu trước khi dùng

Có ít nhất 1 trong các agent sau đã được cài và đăng nhập:

| Agent               | Cách cài                                   | File cần có                               |
| ------------------- | ------------------------------------------ | ----------------------------------------- |
| **Claude Code CLI** | `npm install -g @anthropic-ai/claude-code` | `~/.claude/.credentials.json`             |
| **Cursor**          | Tải từ cursor.sh                           | `state.vscdb` đã có cursorAuth keys       |
| **Windsurf**        | Tải từ codeium.com                         | `state.vscdb` đã có windsurfAuthStatus    |
| **Antigravity**     | Tải từ trang chính thức                    | `state.vscdb` đã có antigravityAuthStatus |

App tự động phát hiện agent nào đã cài và chỉ hiển thị những cái đang có.

> **Lưu ý:** Hiện app **chỉ hiển thị Claude Code**. **Cursor, Windsurf và Antigravity** (Desktop / IDE / CLI) đang **tạm ẩn khỏi giao diện** để khắc phục các lỗi còn tồn đọng — mã nguồn vẫn giữ nguyên và có thể bật lại trong các phiên bản sau. Các bước liên quan tới Cursor / Windsurf / Antigravity bên dưới áp dụng khi chúng được bật lại.

---

## Lần đầu sử dụng

### Bước 1: Cài đặt

Tải file cài đặt phù hợp với hệ điều hành từ [GitHub Releases](https://github.com/ducdev2k1/agent-switch-tools/releases):

| OS      | File                    |
| ------- | ----------------------- |
| Windows | `.msi` hoặc `.exe`      |
| macOS   | `.dmg`                  |
| Linux   | `.deb` hoặc `.AppImage` |

### Bước 2: Chọn Agent và lưu tài khoản đầu tiên

1. Mở Agent Switch Tools
2. Chọn tab tương ứng với agent bạn dùng: **Claude Code**, **Cursor**, **Windsurf**, hoặc **Antigravity**
3. App tự động phát hiện tài khoản đang active trong agent đó
4. Nhấn **"Save Current Profile"** để lưu thành profile
5. Đặt tên cho profile (mặc định là email)

### Bước 3: Thêm tài khoản thứ hai

**Với Claude Code:**

1. **Đăng xuất** Claude Code hiện tại: `claude logout`
2. **Đăng nhập** tài khoản mới: `claude login`
3. Quay lại Agent Switch Tools → nhấn **"Save Current Profile"** lần nữa

**Với IDE (Cursor/Windsurf/Antigravity):**

1. Trong IDE, sign out tài khoản hiện tại
2. Sign in tài khoản mới
3. Quay lại Agent Switch Tools → tab tương ứng → **"Save Current Profile"**

Giờ bạn có 2 profile cho agent đó!

---

## Thao tác hàng ngày

### Chuyển đổi tài khoản

**Cách 1: Từ Dashboard**

1. Mở app → thấy danh sách profiles
2. Nhấn nút **"Switch"** trên profile muốn dùng
3. Xác nhận trong dialog → Done!

**Cách 2: Từ System Tray**

1. Nhấp chuột phải vào icon Agent Switch Tools ở System Tray
2. Chọn profile muốn dùng
3. Xác nhận → Done!

### Xem Quota

Quota hiển thị trực tiếp trên mỗi profile card:

```
┌─────────────────────────────────────┐
│  user@gmail.com           [Switch]  │
│  Pro · Active                       │
│                                     │
│  5h Usage    ████████░░░░  65%      │
│  7d Usage    ██████░░░░░░  48%      │
│  7d Sonnet   ███░░░░░░░░░  22%      │
│                                     │
│  Resets in: 2h 15m                  │
└─────────────────────────────────────┘
```

- Quota tự động cập nhật mỗi 5 phút
- Nhấn nút refresh để cập nhật ngay

---

## Cài đặt (Settings)

### General (Cài đặt chung)

| Tùy chọn    | Mô tả                                 |
| ----------- | ------------------------------------- |
| Auto Update | Tự động kiểm tra và thông báo bản mới |
| Auto Start  | Tự khởi động cùng hệ điều hành        |
| Language    | Chọn Tiếng Anh hoặc Tiếng Việt        |
| Theme       | Light / Dark / System                 |

### Phiên tự động (Auto Session)

Tự động "mở" cửa sổ hạn mức 5 giờ của Claude vào đúng giờ bạn chọn mỗi ngày, để cửa sổ trùng với giờ làm việc (xem giải thích tại [Tính năng → Phiên tự động](tinh-nang.md)).

**Các bước:**

1. Mở **Cài đặt → Phiên tự động**.
2. Đặt **giờ** (HH:MM) bạn muốn bắt đầu phiên — thường là lúc bắt đầu làm việc.
3. Bật công tắc cho **từng tài khoản**, hoặc nhập giờ rồi bấm **"Bật tất cả"** để áp dụng cho mọi tài khoản cùng lúc.
4. Bật **"Khởi động cùng hệ thống"** ở tab Chung để app luôn chạy và prime đúng giờ.

**Mẹo:**

- Bấm **"Prime ngay"** trên một tài khoản để mở cửa sổ thủ công ngay lập tức.
- Nếu cửa sổ 5 giờ đang chạy, thao tác sẽ hiển thị *Hold* (bỏ qua) — đây là hành vi đúng, tránh phá cửa sổ hiện tại.
- Xem **thống kê theo ngày** và **nhật ký hoạt động** ngay trong tab để kiểm tra lịch sử.

> Lưu ý: bộ lập lịch chỉ chạy khi app đang mở. Chỉ tốn 1 token mỗi lần prime.

### Webhook

Cấu hình để app gửi báo cáo usage tới endpoint bên ngoài:

| Field               | Mô tả                                           |
| ------------------- | ----------------------------------------------- |
| URL                 | Địa chỉ webhook (HTTPS, hoặc localhost cho dev) |
| Secret              | Token xác thực (gửi kèm header)                 |
| Trigger             | Manual / On Startup / On Change                 |
| Include Credentials | Có đính kèm credentials trong payload không     |

**Dùng để**: Admin/team lead theo dõi quota usage của nhiều người, hoặc log ra dashboard riêng.

### Session Usage Webhook

Gửi thống kê token đã dùng trong các phiên Claude Code:

| Field        | Mô tả                                                   |
| ------------ | ------------------------------------------------------- |
| URL          | Địa chỉ webhook                                         |
| Period       | Khoảng thời gian thống kê (1h / 5h / 24h / 7d)          |
| Detail Level | Summary (chỉ tổng) hoặc Detailed (bao gồm từng session) |

### Device

- Xem Device ID (không thể thay đổi)
- Đặt tên thiết bị (hiển thị trong webhook payload)

---

## Các tình huống thường gặp

### Token hết hạn

App tự động refresh token ngầm (gọi thẳng OAuth Anthropic) mỗi khi cập nhật usage — bạn thường không cần làm gì. Trên tài khoản đã hết hạn, bạn cũng có thể bấm nút **🔑 (làm mới token)** để refresh ngay; nếu thất bại app sẽ hiện thông báo lỗi cụ thể.

Nếu refresh báo lỗi `invalid_grant` (refresh token đã hết hiệu lực thật):

1. Chạy `claude login` trong terminal để đăng nhập lại tài khoản đó
2. Quay lại Agent Switch Tools → Save profile lại

### Muốn xóa 1 profile

1. Nhấn icon thùng rác trên profile card
2. Xác nhận xóa
3. File trong `~/.agent-switch-tools/{agent}/profiles/{name}/` sẽ bị xóa

**Lưu ý**: Xóa profile KHÔNG ảnh hưởng tới tài khoản agent gốc. Bạn luôn có thể đăng nhập lại và lưu profile mới.

### Cập nhật ứng dụng

- Nếu bật Auto Update: App tự thông báo khi có bản mới
- Nếu tắt: Tải bản mới từ GitHub Releases và cài đè

### Windows hiện cảnh báo SmartScreen

Đây là hành vi bình thường với app mã nguồn mở chưa ký số:

1. Nhấn **"More info"**
2. Nhấn **"Run anyway"**

Hoặc dùng file `.msi` thay `.exe` để ít bị cảnh báo hơn.

---

## Phím tắt / Mẹo

- **Tray icon** luôn chạy ngầm → chuyển đổi profile cực nhanh mà không cần mở app
- **Webhook + On Change** = tự động báo cáo quota mỗi 5 phút → dùng cho dashboard monitoring
- Profile **tự động backup** trước mỗi lần switch → không lo mất data

---

**Xem thêm**: [Tổng quan dự án](tong-quan.md) | [Tính năng chi tiết](tinh-nang.md) | [Tương tác với Claude](tuong-tac-voi-claude.md)
