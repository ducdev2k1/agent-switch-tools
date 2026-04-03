# Release Notes v1.0.2

## 🚀 Tính năng & Cập nhật mới (Features & Updates)

### 1. Tích hợp API Quản lý Hạn mức sử dụng (Usage Limits API)

- **Kiểm soát thông lượng:** Chính thức kết nối hệ thống Backend (Rust) với API OAuth của Anthropic để trực tiếp lấy dữ liệu về hạn mức sử dụng của từng tài khoản.
- **Phân loại giới hạn thông minh:** Hệ thống hỗ trợ xử lý và phân tách chi tiết các báo cáo dữ liệu theo từng gói:
  - Giới hạn theo phiên: Số phiên có thể mở trong vòng vài giờ tới.
  - Giới hạn 7 ngày: Khối lượng công việc chung của tài khoản trong tuần.
  - Giới hạn Sonnet 7 ngày: Giới hạn riêng biệt đối với các model thuộc dòng Sonnet.

### 2. Quản lý Giao diện hiển thị Hạn mức (Frontend Components)

- **Tạo mới Component `UsageLimitsDisplay`:** Đây là component chuyên biệt để trực quan hóa dữ liệu "usage" được backend cung cấp.
- **Giao diện Trải nghiệm người dùng:**
  - Cung cấp thanh Progress Bar (thanh tiến trình) trực quan để hiển thị % đã sử dụng trên từng loại giới hạn (Session/Weekly).
  - Tự động thay đổi màu sắc của thanh tiến trình để cảnh báo người dùng khi hạn mức sắp cạn.
  - Hiển thị trực tiếp thời gian đếm ngược chi tiết (ví dụ: "Đặt lại sau 1 giờ") trên thẻ Account, giúp người dùng sắp xếp công việc chủ động hơn.
- Cập nhật trực tiếp `ProfileCard` để gắn và hiển thị real-time các số liệu này.

---

_Phiên bản này mang tới tính năng quan trọng nhất giúp cảnh báo cho khách hàng tránh bị gián đoạn công việc do chạm ngưỡng giới hạn của API!_
