# Claude Tools v1.0.8 - Ghi chú phát hành

**Claude Tools v1.0.8** bổ sung theo dõi usage theo thiết bị, gửi báo cáo session qua webhook, và tự động làm mới quota cho tất cả tài khoản.

## Có gì mới?

### 1. Định danh thiết bị (Device Identity)

- **UUID duy nhất**: Mỗi máy tạo một `device_id` (UUID v4) khi lần đầu khởi động, lưu tại `~/.claude/.claude-tools/device.json`. ID này không bao giờ thay đổi.
- **Đặt tên thiết bị**: Vào Settings > Device để xem Device ID, hostname và đổi tên thiết bị (mặc định là hostname).
- **Đính kèm vào webhook**: Thông tin thiết bị tự động gắn kèm khi gửi session usage webhook, giúp phân biệt máy nào gửi dữ liệu.

### 2. Theo dõi Session Usage

- **Đọc JSONL sessions**: Backend (Rust) quét tất cả file `.jsonl` trong `~/.claude/projects/`, tổng hợp input/output tokens, cache read/write, số lượng message cho từng session.
- **Xem trước (Preview)**: Nhấn "Preview" trong Settings để xem danh sách sessions với token usage trước khi gửi.
- **Gửi webhook**: Chọn khoảng thời gian (1h, 5h, 24h, 7d) và mức chi tiết (summary hoặc detailed), nhấn gửi để đẩy báo cáo đến webhook URL đã cấu hình. Payload bao gồm: thông tin thiết bị, email thành viên, tổng hợp và chi tiết từng session.

### 3. Tự động làm mới Quota cho tất cả tài khoản

- **Background worker**: Mỗi 5 phút, app tự động gọi Anthropic Usage API để cập nhật quota cho tất cả profiles (active + saved), không cần người dùng bấm refresh.
- **Event-driven**: Frontend nhận event `all-profiles-usage-updated` và `usage-updated` để cập nhật UI realtime, không cần polling.
- **Rate-limit safe**: Delay 1 giây giữa các API call để tránh bị rate-limit.

### 4. Cải tiến CI/CD

- **Build song song**: Workflow release tách bước tạo draft release riêng, cho phép các job build (Ubuntu, macOS Intel, macOS ARM, Windows) chạy đồng thời. Giảm tổng thời gian release.

---

_v1.0.8 giúp bạn theo dõi token usage chi tiết theo từng thiết bị và session, đồng thời quota luôn cập nhật tự động không cần thao tác thủ công._
