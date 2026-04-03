# Release Notes v1.0.4

## 🚀 Tính năng & Cập nhật mới (Features & Updates)

### 1. Giới thiệu Trang Cài đặt (Settings Page)

- **Trang Quản Trị Cấu Hình Mới:** Tích hợp thành công giao diện Cài đặt (Settings) chuyên biệt, có thể mở trực tiếp từ bảng điều khiển chính bằng biểu tượng ⚙️ (Bánh răng).
- **Giao diện Menu Cột Bên (Sidebar):** Màn hình Cài đặt được chia thành các tab quản lý chuyên mục hợp lí và rõ ràng: `Chung` (General), `Webhook` và `Giới thiệu` (About).

### 2. Tính năng bắn dữ liệu Webhook (Webhook Integrations)

- **Cấu hình Endpoint Tuỳ Chỉnh:** Người dùng hiện tại có thể cung cấp URL Endpoint cá nhân hóa để hệ thống tự động bắn dữ liệu Mức Sử Dụng (Usage Reports). Tính năng giải quyết nhu cầu báo cáo số liệu từ nhiều hồ sơ quản trị khác nhau về một máy chủ trung tâm.
- **Tuỳ chọn kích hoạt (Triggers):** 
  - Gắn kèm cơ chế gửi báo cáo tự động khi ứng dụng khởi động.
  - Gắn kèm cơ chế gửi thông báo tự động mỗi khi có sự thay đổi hạn ngạch/mức sử dụng.
  - Gửi dữ liệu dưới dạng thủ công thông qua một nút "Gửi Ngay".
- **Bảo Mật Bổ Sung:** Bổ sung trường thiết lập mã xác thực Auth Secret cá nhân hóa. Nếu được kết nối, hệ thống sẽ chèn token này và Header `Authorization: Bearer <Secret>` của các Webhook Request.
- **Tiện ích Kiểm Tra (Test Connection):** Cung cấp công cụ Test mô phỏng thử nghiệm với Endpoint, giúp bạn chắc chắn rằng liên kết dữ liệu giữa ứng dụng và máy chủ đang hoạt động chuẩn xác trước khi đưa vào vận hành.

### 3. Settings: Cài đặt Chung (General Preferences)

- **Chủ đề Giao diện (Theme Modes):** Thay đổi giao diện trực tiếp trong Setting panel, cung cấp đầy đủ các chế độ hiển thị Sáng, Tối và tuỳ chỉnh theo cấu hình máy tính (System).
- **Quản lý Bản Cập Nhật (Auto Updates):** Tích hợp công tắc bật/tắt để kiểm tra các phiên bản mới một cách chủ động.
- **Thông tin Phiên Bản (About):** Hợp nhất các thông tin như version, giấy phép nguồn mở, mã lực từ nhà phát triển... vào chuyên mục About.

### 4. Nâng Cấp Hệ Thống Đa Ngôn Ngữ (i18n Enhancements)

- Đồng bộ hóa 100% tất cả các từ khóa và nhãn hiển thị bên trong toàn bộ Trang Cài Đặt mới vào kho lưu trữ nội địa hóa (`vi.json`, `en.json`), tuân thủ triệt để nguyên tắc ưu tiên hệ ngôn ngữ tiếng Việt của hệ thống.

### 5. Bump Phiên Bản (Version Maintenance)

- Tự động thay đổi bản cập nhật lên mức `1.0.4` đối với gói lõi `package.json` và cấu hình hệ sinh thái `tauri.conf.json`.

---

_Bản cập nhật v1.0.4 mang lại công năng truyền thông dữ liệu mạnh mẽ với Webhook, cho phép ứng dụng trở thành trạm điều khiển các tài khoản linh hoạt có khả năng móc nối, giao tiếp báo cáo lên mọi Data Center._
