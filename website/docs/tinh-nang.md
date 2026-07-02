---
sidebar_position: 2
title: Tính Năng Chi Tiết
---

> **Lưu ý:** Hiện tại app **chỉ hiển thị Claude Code**. **Cursor, Windsurf và Antigravity** (Desktop / IDE / CLI) đang **tạm ẩn khỏi giao diện** để khắc phục các lỗi còn tồn đọng — mã nguồn vẫn giữ nguyên đầy đủ và có thể bật lại trong các phiên bản sau. Các phần mô tả Cursor / Windsurf / Antigravity bên dưới vẫn đúng về mặt kỹ thuật và sẽ hoạt động trở lại khi được bật.

## 1. Quản lý đa tài khoản (Profile Management)

### Vấn đề

Mỗi AI coding agent (Claude Code, Cursor, Windsurf, Antigravity) chỉ cho phép đăng nhập **1 tài khoản** tại một thời điểm. Muốn dùng tài khoản khác, bạn phải đăng xuất rồi đăng nhập lại.

### Giải pháp

Agent Switch Tools tạo một hệ thống **profile** thống nhất — mỗi agent có kho profile riêng biệt với cấu trúc đồng nhất:

```
~/.agent-switch-tools/
├── device.json                 ← Định danh thiết bị (toàn cục)
├── claude/                     ← Claude Code data
│   ├── meta.json               ← Profile đang active + lịch sử
│   └── profiles/
│       ├── email1@gmail.com/
│       │   ├── credentials.json    ← Backup credentials
│       │   └── oauth.json          ← Thông tin OAuth
│       └── email2@company.com/
│           ├── credentials.json
│           └── oauth.json
├── cursor/                     ← Cursor IDE data
│   └── profiles/
│       └── {email}/
│           └── auth-backup.json    ← Backup auth keys từ state.vscdb
├── windsurf/                   ← Windsurf IDE data
│   └── profiles/{email}/
├── antigravity/                ← Antigravity Desktop data
│   └── profiles/{email}/
├── antigravity-ide/            ← Antigravity IDE data
│   └── profiles/{email}/
└── antigravity-cli/            ← Antigravity CLI data
    └── profiles/{email}/
```

### Tự động migrate từ cấu trúc cũ

App tự chuyển dữ liệu từ các vị trí cũ khi khởi động (chỉ bù phần còn thiếu, không ghi đè dữ liệu mới hơn):

- `~/.claude/.claude-tools/` (v1.0.8–v1.0.9)
- `~/.claude-tools/` (phiên bản trung gian)
- File phẳng trong `~/.claude/` (phiên bản rất cũ)
- **Từ v1.0.13**: app từng được đổi tên (`claude-tools` → `agent-switch-tools`) kèm dời thư mục dữ liệu sang `~/.agent-switch-tools/`. Dữ liệu kẹt ở vị trí cũ (profiles, lịch sử chuyển tài khoản, device identity) được tự khôi phục một lần khi mở app.

### Các thao tác

| Thao tác            | Mô tả                                                                |
| ------------------- | -------------------------------------------------------------------- |
| **Thêm profile**    | Lưu credentials hiện tại thành 1 profile mới                         |
| **Chuyển đổi**      | Hoán đổi (swap) file credentials giữa profile active và profile đích |
| **Đổi tên**         | Đổi tên thư mục profile                                              |
| **Xóa**             | Xóa thư mục profile (có hộp thoại xác nhận)                          |
| **Sao lưu tự động** | Khi chuyển profile, credentials hiện tại tự động được backup trước   |

### Cơ chế chuyển đổi (Switch)

**Với Claude Code** (file-based):

```
Trước khi switch:
  ~/.claude/.credentials.json       ← tài khoản A (đang active)
  ~/.agent-switch-tools/claude/profiles/B/credentials.json  ← B (đã lưu)

Khi nhấn "Switch to B":
  1. Backup A: copy .credentials.json → profiles/A/credentials.json
  2. Restore B: copy profiles/B/credentials.json → .credentials.json
  3. Cập nhật meta.json: active = B
```

**Với IDE (Cursor/Windsurf/Antigravity)** (SQLite-based) — _hiện tạm ẩn khỏi giao diện, mô tả dưới đây áp dụng khi bật lại_:

```
Trước khi switch:
  IDE state.vscdb (ItemTable)           ← tài khoản A (đang active)
  profiles/B/auth-backup.json           ← B (đã lưu)

Khi nhấn "Switch to B":
  1. Đọc auth keys hiện tại từ state.vscdb, backup sang profiles/A/
  2. Ghi auth keys của B vào state.vscdb (UPDATE ItemTable)
  3. Cập nhật meta.json: active = B
  4. (Tùy chọn) Khởi động lại IDE để nhận tài khoản mới
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

#### Quota Antigravity (cả 3 biến thể) — cập nhật từ v1.0.12, tạm ẩn từ v1.0.13

Theo chính sách Gemini mới, Antigravity tính quota theo **giới hạn Weekly + 5 giờ cho từng nhóm model**. App gọi endpoint mới (đúng cái lệnh `usage` native dùng):

> **Lưu ý:** Từ v1.0.13, Antigravity được tạm ẩn khỏi giao diện để khắc phục lỗi. Mã nguồn tích hợp quota này vẫn giữ nguyên.

```
POST https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary
Body: {}
Headers: Authorization: Bearer {access_token}, User-Agent: Antigravity/...
```

- Hiển thị 4 bucket: **Gemini — Weekly / 5h** và **Claude and GPT — Weekly / 5h** (theo **% còn lại**).
- Token tự refresh qua Google OAuth khi hết hạn; email lấy qua Google `userinfo` (cho IDE/CLI).
- Nếu tài khoản Google chưa xác minh (vd chưa có SĐT) → API trả `403 "Verify your account"`, không có quota — đúng như CLI native.

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

Agent Switch Tools đặt một icon ở System Tray. Khi nhấp chuột phải, menu hiển thị từng section cho mỗi agent đã cài:

```
┌────────────────────────────┐
│ Agent Switch Tools          │  ← Header
│ ──────────────────────────  │
│ ✓ claude@gmail.com (active) │  ← Claude Code active
│   work@company.com          │  ← Profile đã lưu (nhấp để switch)
│ ──────────────────────────  │
│ Cursor                      │  ← Header IDE
│ ✓ user@cursor.sh (active)   │  ← Cursor active
│   team@cursor.sh            │
│ ──────────────────────────  │
│ Windsurf                    │  ← Header IDE (nếu đã cài)
│ ✓ user@codeium.com (active) │
│ ──────────────────────────  │
│   Open Dashboard            │
│   Quit                      │
└────────────────────────────┘
```

- Chỉ IDE **đã cài** mới xuất hiện trong menu
- Nhấp vào profile bất kỳ → **switch ngay trong nền** — không hộp thoại xác nhận, không bật cửa sổ app, hoạt động kể cả khi dashboard đang đóng
- Với IDE: chuyển xong tự khởi động lại IDE nếu nó đang chạy

---

## 4. Refresh Token tự động

### Vấn đề

Access token trong `credentials.json` có thời hạn ngắn (~8 giờ). Với tài khoản đang active, Claude Code CLI tự refresh khi chạy. Nhưng các profile **không active** thì không được refresh → token có thể hết hạn ngay lúc bạn muốn chuyển sang.

### Giải pháp

App refresh token **trực tiếp qua endpoint OAuth của Anthropic** (`https://claude.ai/v1/oauth/token`) — **không** chạy `claude` CLI, **không** tốn quota, **không** hoán đổi credentials:

1. Đọc `refreshToken` từ file credentials của profile.
2. Gọi Anthropic để lấy access token + refresh token mới (token xoay vòng — token cũ bị vô hiệu).
3. Ghi đè 3 trường (`accessToken`, `refreshToken`, `expiresAt`) **trực tiếp vào file của đúng profile đó**, ghi an toàn (atomic) và **không đụng tới tài khoản đang active**.

Cơ chế này chạy ở 3 nơi:

- **Tự động khi lấy usage** (gồm cả worker nền mỗi 5 phút): nếu token sắp hết hạn thì tự refresh; lỗi thì giữ token cũ (best-effort, không chặn việc xem usage).
- **Nút "làm mới token" (🔑) thủ công** trên tài khoản hết hạn: refresh ép buộc, hiện thông báo lỗi rõ ràng nếu thất bại.
- **Phiên tự động (priming)**: refresh trước khi mở cửa sổ 5 giờ.

> Nếu refresh báo lỗi `invalid_grant`, refresh token của tài khoản đó đã hết hiệu lực thật — cần đăng nhập lại tài khoản đó.

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
  "app_version": "1.0.10",
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

## 10. Phiên tự động (Auto Session / Priming)

### Vấn đề

Claude tính hạn mức theo **cửa sổ 5 giờ**: đồng hồ 5 giờ bắt đầu chạy kể từ **lần đầu tiên bạn dùng** trong giai đoạn đó. Nếu bạn lỡ kích hoạt vào lúc không định làm việc (ví dụ 6h sáng), cửa sổ sẽ reset vào 11h trưa — lệch hẳn với giờ làm thực tế và lãng phí phần lớn quota.

### Giải pháp

Phiên tự động sẽ **tự "mở" cửa sổ 5 giờ vào đúng giờ bạn chọn** mỗi ngày, bằng cách gửi một tin nhắn cực nhỏ (`"hi"`, tối đa 1 token) tới Claude. Nhờ đó cửa sổ 5 giờ trùng với giờ làm việc của bạn, tận dụng tối đa hạn mức.

### Cách hoạt động

- Mỗi tài khoản có một **công tắc bật/tắt** và một **giờ hẹn** (HH:MM).
- Bộ lập lịch chạy nền **mỗi phút**: khi tới giờ hẹn và tài khoản **chưa prime trong ngày**, app gửi tin nhắn mở cửa sổ.
- Sau khi gửi, app **kiểm chứng** cửa sổ mới đã thực sự mở (đồng hồ reset nhảy sang mốc tương lai mới) rồi mới báo thành công.
- Tốn đúng **1 token** trên model Haiku giá rẻ — gần như không ảnh hưởng hạn mức.
- Nếu một cửa sổ 5 giờ **đang chạy sẵn**, app **bỏ qua** (trạng thái *Hold*) để không phá cửa sổ hiện tại.
- Mỗi tài khoản chỉ prime **một lần mỗi ngày**.

### Lưu ý quan trọng

Bộ lập lịch **chỉ chạy khi app đang mở**. Để prime đúng giờ kể cả khi bạn chưa mở app thủ công, hãy bật **"Khởi động cùng hệ thống"** trong tab Cài đặt → Chung.

### Theo dõi

Tab **Phiên tự động** còn hiển thị **thống kê theo ngày** (thành công / hold / lỗi / bỏ qua) và **bảng nhật ký hoạt động** (Thời gian / Tài khoản / Trạng thái / Chi tiết, ngày giờ dd/mm/yyyy hh:mm) để bạn kiểm tra lịch sử prime.

> Xem hướng dẫn thao tác từng bước tại [Hướng dẫn sử dụng → Phiên tự động](huong-dan-su-dung.md#phiên-tự-động-auto-session).

---

**Tiếp theo**: [Tương tác với Claude — Giải thích kỹ thuật](tuong-tac-voi-claude.md)
