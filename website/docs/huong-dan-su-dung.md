---
sidebar_position: 5
title: Hướng Dẫn Sử Dụng
---

# Hướng Dẫn Sử Dụng

## Yêu cầu trước khi dùng

1. **Đã cài Claude Code CLI** và đã đăng nhập ít nhất 1 tài khoản
2. File `~/.claude/.credentials.json` phải tồn tại (Claude Code tự tạo khi đăng nhập)

---

## Lần đầu sử dụng

### Bước 1: Cài đặt

Tải file cài đặt phù hợp với hệ điều hành từ [GitHub Releases](https://github.com/ducdev2k1/claude-tools/releases):

| OS | File |
|---|---|
| Windows | `.msi` hoặc `.exe` |
| macOS | `.dmg` |
| Linux | `.deb` hoặc `.AppImage` |

### Bước 2: Lưu tài khoản đầu tiên

1. Mở Claude Tools
2. App tự động phát hiện tài khoản Claude Code đang active
3. Nhấn **"Save Current Profile"** để lưu thành profile
4. Đặt tên cho profile (mặc định là email)

### Bước 3: Thêm tài khoản thứ hai

1. **Đăng xuất** Claude Code CLI hiện tại: `claude logout`
2. **Đăng nhập** tài khoản mới: `claude login`
3. Quay lại Claude Tools → nhấn **"Save Current Profile"** lần nữa
4. Giờ bạn có 2 profile!

---

## Thao tác hàng ngày

### Chuyển đổi tài khoản

**Cách 1: Từ Dashboard**
1. Mở app → thấy danh sách profiles
2. Nhấn nút **"Switch"** trên profile muốn dùng
3. Xác nhận trong dialog → Done!

**Cách 2: Từ System Tray**
1. Nhấp chuột phải vào icon Claude Tools ở System Tray
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

### General

| Tùy chọn | Mô tả |
|---|---|
| Auto Update | Tự động kiểm tra bản mới |
| Auto Start | Tự khởi động cùng OS |
| Language | Tiếng Anh / Tiếng Việt |
| Theme | Light / Dark / System |

### Webhook

Cấu hình gửi báo cáo usage tới endpoint bên ngoài (cho admin/team lead theo dõi).

### Device

Xem Device ID và đặt tên thiết bị.

---

## Các tình huống thường gặp

### Token hết hạn

App tự động phát hiện và refresh token ngầm. Nếu refresh thất bại:
1. Chuyển sang profile đó (Switch)
2. Chạy `claude login` trong terminal
3. Quay lại Claude Tools → Save profile lại

### Muốn xóa 1 profile

1. Nhấn icon thùng rác trên profile card
2. Xác nhận xóa

**Lưu ý**: Xóa profile KHÔNG ảnh hưởng tới tài khoản Claude.

### Windows hiện cảnh báo SmartScreen

1. Nhấn **"More info"**
2. Nhấn **"Run anyway"**

Hoặc dùng file `.msi` thay `.exe`.

---

## Mẹo

- **Tray icon** luôn chạy ngầm → chuyển đổi profile cực nhanh
- **Webhook + On Change** = tự động báo cáo quota mỗi 5 phút
- Profile **tự động backup** trước mỗi lần switch
