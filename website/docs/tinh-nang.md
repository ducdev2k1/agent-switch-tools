---
sidebar_position: 2
title: Tính Năng Chi Tiết
---

# Tính Năng Chi Tiết

## 1. Quản lý đa tài khoản (Profile Management)

### Vấn đề
Claude Code CLI chỉ cho phép đăng nhập **1 tài khoản** tại một thời điểm. Thông tin đăng nhập được lưu tại `~/.claude/.credentials.json`. Muốn dùng tài khoản khác, bạn phải đăng xuất rồi đăng nhập lại.

### Giải pháp
Claude Tools tạo một hệ thống **profile** — mỗi tài khoản được lưu riêng trong một thư mục:

```
~/.claude/.claude-tools/
├── profiles/
│   ├── email1@gmail.com/
│   │   ├── credentials.json    ← Bản backup credentials
│   │   └── oauth.json          ← Thông tin OAuth (email, subscription...)
│   ├── email2@company.com/
│   │   ├── credentials.json
│   │   └── oauth.json
│   └── ...
├── meta.json                   ← Profile đang active + lịch sử chuyển đổi
└── device.json                 ← Thông tin thiết bị
```

### Các thao tác

| Thao tác | Mô tả |
|---|---|
| **Thêm profile** | Lưu credentials hiện tại thành 1 profile mới |
| **Chuyển đổi** | Hoán đổi (swap) file credentials giữa profile active và profile đích |
| **Đổi tên** | Đổi tên thư mục profile |
| **Xóa** | Xóa thư mục profile (có hộp thoại xác nhận) |
| **Sao lưu tự động** | Khi chuyển profile, credentials hiện tại tự động được backup trước |

### Cơ chế chuyển đổi (Switch)

```
Trước khi switch:
  ~/.claude/.credentials.json  ← tài khoản A (đang active)
  profiles/B/credentials.json  ← tài khoản B (đã lưu)

Khi nhấn "Switch to B":
  1. Backup A: copy .credentials.json → profiles/A/credentials.json
  2. Restore B: copy profiles/B/credentials.json → .credentials.json
  3. Cập nhật meta.json: active = B

Sau khi switch:
  ~/.claude/.credentials.json  ← tài khoản B (giờ là active)
  profiles/A/credentials.json  ← tài khoản A (đã lưu)
```

---

## 2. Theo dõi hạn mức sử dụng (Quota Monitoring)

### Quota là gì?

Mỗi tài khoản Claude có giới hạn sử dụng (quota) theo thời gian:
- **5 giờ**: Giới hạn sử dụng trong 5 giờ gần nhất
- **7 ngày**: Giới hạn sử dụng trong 7 ngày
- **7 ngày (Sonnet)**: Giới hạn riêng cho model Sonnet trong 7 ngày

### Cách hoạt động

App gọi **Anthropic OAuth API** để lấy thông tin quota:

```
GET https://api.anthropic.com/api/oauth/usage
Headers:
  Authorization: Bearer {accessToken từ credentials.json}
  anthropic-beta: oauth-2025-04-20
```

**Kết quả trả về**: % đã sử dụng và thời gian reset cho mỗi loại quota.

### Tự động cập nhật (Background Worker)

- Một worker chạy ngầm mỗi **5 phút**
- Fetch quota cho **tất cả profile** (active + saved)
- Mỗi profile cách nhau 1 giây (tránh spam API)
- Cache kết quả **2 phút** — không gọi API lại nếu data còn mới

---

## 3. Chuyển đổi nhanh từ System Tray

System Tray (khay hệ thống) là khu vực nhỏ ở góc phải phía dưới thanh taskbar (Windows/Linux) hoặc thanh menu bar (macOS).

Claude Tools đặt một icon ở System Tray. Khi nhấp chuột phải:

```
┌────────────────────────┐
│ ✓ email1@gmail.com     │  ← Profile đang active (có dấu ✓)
│   email2@company.com   │  ← Profile đã lưu (nhấp để switch)
│   email3@org.com       │
│ ──────────────────────  │
│   Open Dashboard       │  ← Mở cửa sổ chính
│   Quit                 │  ← Thoát ứng dụng
└────────────────────────┘
```

---

## 4. Refresh Token tự động

Khi phát hiện token sắp hết hạn, app sẽ:

1. Tạm hoán đổi credentials sang profile cần refresh
2. Chạy lệnh: `claude -p "hi" --max-turns 1` (Claude CLI sẽ tự refresh token)
3. Copy file credentials đã refresh ngược về profile
4. Hoán đổi lại credentials gốc

Quá trình này diễn ra ngầm, không ảnh hưởng tới công việc hiện tại.

---

## 5. Webhook — Gửi báo cáo tự động

### 2 loại Webhook

#### A. Usage Report Webhook

Gửi thông tin quota của tất cả profile. **Trigger modes**: Thủ công | Khi khởi động app | Mỗi khi quota thay đổi (mỗi 5 phút)

#### B. Session Usage Webhook

Gửi thống kê token đã dùng trong các phiên Claude Code. **Periods**: 1 giờ | 5 giờ | 24 giờ | 7 ngày

---

## 6. Tự động cập nhật (Auto Update)

- App kiểm tra GitHub Releases để tìm phiên bản mới
- Khi có bản mới → hiện modal thông báo (chỉ hiện 1 lần mỗi version)
- Người dùng chọn "Cập nhật" → tải, cài đặt, khởi động lại tự động

---

## 7. Đa ngôn ngữ (i18n) & Giao diện

- **Tiếng Anh** (en) và **Tiếng Việt** (vi)
- **Light / Dark / System** mode

---

## 8. CLI Status Bar

Hiển thị trạng thái Claude Code CLI hiện tại: Model, Sessions, .env status.

---

## 9. Device Tracking

Mỗi máy tính được gán Device ID, Device Name, Hostname — đính kèm trong webhook payload để phân biệt máy.
