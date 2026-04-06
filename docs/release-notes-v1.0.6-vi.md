# Nhật ký phát hành Claude Tools v1.0.6

Chào mừng bạn đến với bản cập nhật **v1.0.6** của **Claude Tools**. Bản phát hành này tập trung vào sự tinh chỉnh hệ thống theo dõi lưu lượng (quota), đảm bảo dữ liệu sử dụng chi tiết và thời gian thực nhất cho các cấu hình (profiles) đang hoạt động.

## Có gì mới?

### 1. Nâng cấp hệ thống theo dõi lưu lượng (Quota) thời gian thực

- **Truy xuất dữ liệu có phân biệt**: Ứng dụng giờ đây có thể phân biệt chính xác giữa hồ sơ đang hoạt động và hồ sơ lưu trữ khi lấy số liệu thống kê sử dụng.
- **Tăng độ chính xác**: Đối với tài khoản hiện đang được sử dụng chính, hệ thống sẽ ưu tiên các thông tin đăng nhập từ phiên làm việc thực tế, cung cấp độ chính xác tuyệt đối cho số dư tài khoản Anthropic của bạn.

### 2. Tối ưu hóa Giao diện và Hooks (Frontend)

- **Cải thiện `useProfileUsage` Hook**: Hook đã được cập nhật để hỗ trợ đồng bộ hóa trạng thái `isActive`, đảm bảo giao diện luôn hiển thị dữ liệu mới nhất cho tài khoản đang được sử dụng.
- **Tích hợp `ProfileCard`**: Thành phần hiển thị hồ sơ hiện tận dụng các tham số backend mới để cung cấp cái nhìn tổng quan về trạng thái đáng tin cậy hơn.

### 3. Cải thiện sự ổn định của Backend (Rust)

- **Tối ưu xử lý đường dẫn**: Tinh chỉnh logic định vị dữ liệu xác thực, giảm độ trễ của hệ thống tập tin khi chuyển đổi giữa nhiều tài khoản khác nhau.
- **Kiểm soát tham số chặt chẽ hơn**: Các lệnh nội bộ hiện có tính năng kiểm tra kiểu dữ liệu nghiêm ngặt hơn cho các tham số tùy chọn, giúp cải thiện độ ổn định tổng thể của ứng dụng.

### 4. Cập nhật phiên bản

- Đã cập nhật định danh phiên bản nội bộ lên **1.0.6** trên toàn bộ lõi ứng dụng và tệp cấu hình cho máy tính (tauri.conf.json).

---

_Bản cập nhật v1.0.6 nhắm tới mục tiêu nâng cao độ chính xác tối ưu, đảm bảo rằng những gì bạn thấy trên bảng điều khiển khớp chính xác với dữ liệu mà Claude Code CLI thực sự sử dụng trong các phiên làm việc của bạn._
