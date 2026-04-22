# Ghi chú phát hành Agent Switch Tools v1.0.11

**Agent Switch Tools v1.0.11** mang đến khả năng hiển thị quota đầy đủ cho toàn bộ tài khoản Antigravity (kể cả profile đã lưu, không đang active), bổ sung giờ reset dạng 12h cho Claude CLI, và cache fallback thông minh hơn để UI không bị trống khi API quota gặp sự cố.

## Có gì mới?

### 1. Tự động refresh token — Claude CLI & Antigravity

Cả 2 provider giờ tự refresh access token khi hết hạn, nên saved profile lấy quota được mãi chứ không chỉ profile đang active.

**Claude CLI** — khi `accessToken` còn 5 phút là hết hạn, app POST `refreshToken` đã lưu lên endpoint OAuth của Anthropic (`console.anthropic.com/v1/oauth/token`) và ghi đè `credentials.json` atomic với cặp token mới. Anthropic rotate refresh token mỗi lần đổi, nên file phải được cập nhật liên tục.

**Antigravity** — pattern tương tự qua Google OAuth. Refresh credentials trích từ protobuf blob của IDE tại `antigravityUnifiedStateSync.oauthToken` bằng parser thủ công (không thêm dependency).

### 2. Quota Antigravity — Hoạt động cho tất cả tài khoản, không chỉ tài khoản active

- **Flow OAuth refresh**: Các profile Antigravity đã lưu giờ có thể tự lấy quota. Khi access token của profile hết hạn (token OAuth của Google chỉ sống ~1 giờ), app sẽ đọc refresh token từ protobuf blob đã lưu và ngầm đổi lấy access token mới qua endpoint OAuth của Google.
- **Gộp nhóm theo model**: Thay vì một khối quota rời rạc, dashboard hiện hiển thị 3 bucket khớp với UI native của Antigravity:
  - **Gemini Pro** (3.1 Pro High/Low + 3 Pro High/Low — dùng chung pool rate-limit)
  - **Gemini Flash** (3 Flash + 3.1 Flash Lite + các biến thể khác)
  - **Claude / GPT** (Claude Sonnet/Opus 4.6 + GPT-OSS 120B — pool premium)
- **Semantic đảo ngược**: Bucket Antigravity hiển thị **% còn lại** (100 = quota đầy, 0 = cạn) với màu đảo (xanh lá → đầy, đỏ → sắp hết). Claude CLI vẫn giữ convention % đã dùng.

### 3. Claude CLI — Thời gian reset dạng đồng hồ 12h

- Mỗi hàng usage giờ hiển thị cả đếm ngược tương đối lẫn giờ chính xác — ví dụ `R: 2h 15m (3:45 PM)`. Format theo locale hệ thống (12h AM/PM).

### 4. Xử lý API quota lỗi mượt hơn

- **Stale-cache fallback**: Khi API quota live lỗi (mất mạng, 401 sau khi token bị thu hồi, 5xx), dashboard vẫn giữ progress bar + số liệu lần fetch gần nhất thay vì hiển thị trống. Không còn card trống rỗng sau một hiccup mạng tạm thời.
- Áp dụng cho cả Claude CLI và Antigravity.

### 5. Provider không hỗ trợ được đánh dấu rõ ràng

- Card profile Cursor và Windsurf giờ hiển thị **"Quota không khả dụng"** (text nhạt, in nghiêng) thay vì khoảng trống. Hai IDE này chưa expose public single-user quota API.

### 6. Dọn dẹp backend

- Tái cấu trúc module: `src-tauri/src/ide/` → `src-tauri/src/modules/{core,providers,quota,shared}/` để tách bạch rõ hơn giữa IDE providers, hạ tầng dùng chung, và lệnh core.

---

_v1.0.11 thu hẹp khoảng cách giữa UI native của Antigravity và dashboard Agent Switch Tools — mọi tài khoản IDE của bạn đều quan sát được, bất kể cái nào đang active._
