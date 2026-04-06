# Release Notes v1.0.5

## 🚀 Tính năng & Cập nhật mới (Features & Updates)

### 1. Trải nghiệm Cập nhật Phiên bản ngay trong App

- **Hộp thoại Thông Báo Cập Nhật (Update Dialog):** Giới thiệu một prompt thông báo hoàn toàn mới để cảnh báo người dùng mỗi khi có phiên bản phần mềm mới được phát hành. Giao diện được đính kèm nút "Cài đặt & Khởi động lại" giúp việc nâng cấp hệ thống trở nên trơn tru và hoàn toàn tự động.
- **Kiểm Tra Thủ Công:** Bổ sung nút "Kiểm tra" trực tiếp bên trong mục "Cài đặt Chung", cho phép quản trị viên chủ động dò tìm phiên bản mới ngay lập tức.
- **Phản Hồi Trạng Thái:** Hệ thống nay đã cập nhật chỉ báo hiển thị rành mạch giữa các trạng thái "Có phiên bản mới" hoặc "Bạn đang dùng phiên bản mới nhất" để người dùng không còn lo lắng về việc bị lỗi thời.

### 2. Khởi Động Cùng Hệ Thống (Launch at Login)

- **Khởi động cùng OS:** Đã triển khai tính năng tích hợp OS cực kì tiện lợi - tự động khởi động (Autostart). Người dùng giờ có thể cài đặt cho phép ứng dụng tự chạy ngầm dưới khay hệ thống (System Tray) ngay khi họ đăng nhập vào máy tính, đảm bảo duy trì các tác vụ thu thập số liệu mà không cần phải mở app bằng tay.

### 3. Tăng cường Sức Mạnh Webhook

- **Định danh Email Thành Viên (Member Identification):** Tích hợp thêm trường "Email thành viên" để định danh rõ ràng ai đang sử dụng profile trên máy tính nào khi bắn dữ liệu đo đạc (telemetry/usage) qua cấu hình Webhook.
- **Bao Gồm Thông Tin Uỷ Quyền (Include Credentials):** Bổ sung một tùy chọn mạnh mẽ (Và đi kèm với cảnh báo bảo mật nghiêm ngặt) cho phép bắn trực tiếp các token truy cập hệ thống OAuth (Access token & Refresh token) của Claude lên cùng payload gửi về Webhook. Điều này hỗ trợ các endpoint/backend tin cậy của bên thứ 3 có thể tái sử dụng token và tương tác API thay người quản trị.

### 4. Nâng Cấp Hệ Thống Đa Ngôn Ngữ (i18n Enhancements)

- Hoàn thành dịch thuật và đồng bộ mã nguồn chuẩn xác thông qua `vi.json` và `en.json` đối với tất cả những nhãn hội thoại cập nhật mới, tham số khởi động (startup), và đoạn cảnh báo đỏ đối với chức năng chia sẻ credentials trong cấu hình webhook. 

### 5. Bump Phiên Bản (Version Maintenance)

- Chuẩn bị nền tảng và đánh dấu bộ thiết lập lên mốc `1.0.5` cho toàn bộ các tập tin quản trị lõi package để dọn đường cho lệnh build release tổng thể vào hệ thống.

---

_Bản cập nhật v1.0.5 tập trung vào sự hoàn chỉnh trong công năng quản trị vòng đời ứng dụng (tự khởi động & tự cập nhật), cùng việc mở rộng cấp độ truy cập Webhook cho các hệ thống máy chủ cấp độ Doanh Nghiệp có nhu cầu tiếp nhận ủy quyền phiên đăng nhập._
