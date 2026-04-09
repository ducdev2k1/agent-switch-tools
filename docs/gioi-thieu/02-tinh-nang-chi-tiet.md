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

### System Tray là gì?

System Tray (khay hệ thống) là khu vực nhỏ ở góc phải phía dưới thanh taskbar (Windows/Linux) hoặc thanh menu bar (macOS).

### Cách hoạt động

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

Nhấp vào profile bất kỳ → hiển thị dialog xác nhận → switch ngay lập tức.

---

## 4. Refresh Token tự động

### Vấn đề

Token (mã xác thực) trong credentials.json có thời hạn. Khi hết hạn, Claude Code CLI sẽ tự động refresh token khi chạy. Nhưng các profile **không active** thì không được refresh → token có thể hết hạn.

### Giải pháp

Khi phát hiện token sắp hết hạn, app sẽ:

1. Tạm hoán đổi credentials sang profile cần refresh
2. Chạy lệnh: `claude -p "hi" --max-turns 1` (Claude CLI sẽ tự refresh token)
3. Copy file credentials đã refresh ngược về profile
4. Hoán đổi lại credentials gốc

Quá trình này diễn ra ngầm, không ảnh hưởng tới công việc hiện tại.

---

## 5. Webhook — Gửi báo cáo tự động

### Dùng cho ai?

- **Team lead / Admin** muốn theo dõi quota usage của team
- **Cá nhân** muốn log lịch sử sử dụng ra hệ thống bên ngoài

### 2 loại Webhook

#### A. Usage Report Webhook

Gửi thông tin quota của tất cả profile:

```json
{
  "event": "usage_report",
  "timestamp": "2026-04-09T23:00:00Z",
  "app_version": "1.0.7",
  "device_info": {
    "device_id": "uuid",
    "device_name": "PC của tôi",
    "hostname": "my-laptop"
  },
  "data": {
    "profiles": [
      {
        "name": "email@gmail.com",
        "is_active": true,
        "subscription_type": "pro",
        "usage": {
          "5h": { "utilization": 15.2, "reset": "2h 30m" },
          "7d": { "utilization": 45.0, "reset": "3d 12h" }
        }
      }
    ]
  }
}
```

**Trigger modes**: Thủ công | Khi khởi động app | Mỗi khi quota thay đổi (mỗi 5 phút)

#### B. Session Usage Webhook

Gửi thống kê token đã dùng trong các phiên Claude Code:

```json
{
  "event": "session_usage_report",
  "period": "24h",
  "summary": {
    "total_input_tokens": 123456,
    "total_output_tokens": 654321,
    "session_count": 5
  }
}
```

**Periods**: 1 giờ | 5 giờ | 24 giờ | 7 ngày

---

## 6. Tự động cập nhật (Auto Update)

- App kiểm tra GitHub Releases để tìm phiên bản mới
- Khi có bản mới → hiện modal thông báo (chỉ hiện 1 lần mỗi version)
- Người dùng chọn "Cập nhật" → tải, cài đặt, khởi động lại tự động
- Có thể tắt auto-update trong Settings

---

## 7. Đa ngôn ngữ (i18n) & Giao diện

### Ngôn ngữ

- **Tiếng Anh** (en)
- **Tiếng Việt** (vi) — mặc định

Chuyển đổi trong Settings. Ngôn ngữ được lưu vào `localStorage`.

### Giao diện

- **Light mode** — Nền sáng
- **Dark mode** — Nền tối
- **System** — Theo cài đặt hệ điều hành

---

## 8. CLI Status Bar

Hiển thị trạng thái Claude Code CLI hiện tại:

- **Model**: Claude đang dùng model nào (ví dụ: `claude-sonnet-4-6`)
- **Sessions**: Số phiên làm việc đã có
- **.env status**: File cấu hình có tồn tại không

Thông tin này đọc từ `~/.claude/settings.json` và `~/.claude/history.jsonl`.

---

## 9. Device Tracking

Mỗi máy tính được gán:
- **Device ID**: UUID duy nhất, tạo 1 lần, không bao giờ đổi
- **Device Name**: Tên do người dùng tự đặt
- **Hostname**: Tự động lấy từ hệ thống

Thông tin này được đính kèm trong webhook payload để phân biệt báo cáo từ máy nào.

---

**Tiếp theo**: [Tương tác với Claude — Giải thích kỹ thuật](03-tuong-tac-voi-claude.md)
