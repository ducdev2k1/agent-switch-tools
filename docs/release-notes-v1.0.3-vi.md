# Release Notes v1.0.3

## 🚀 Tính năng & Cập nhật mới (Features & Updates)

### 1. Nâng cấp Hệ thống Làm mới Hạn mức (Smart & Throttled Quota Refresh)
- **Tự động chạy ngầm (Background Worker):** Tích hợp một luồng worker hoàn toàn mới trên Backend (Rust). Cứ mỗi 5 phút, hệ thống sẽ âm thầm gọi API cập nhật dữ liệu Usage mà không cần mở ứng dụng (miễn là App còn chạy ở System Tray).
- **Làm mới thông minh khi chuyển Tab (Focus Refresh):** Mỗi khi quay lại (Focus) vào cửa sổ ứng dụng, giao diện sẽ tự động cập nhật số liệu mới nhất.
- **Chống Spam (Anti-Rate Limit Throttling):** Hệ thống được bảo vệ bởi cơ chế "Throttle" 120 giây (2 phút). Dù bạn có chuyển qua lại giữa các cửa sổ ứng dụng liên tục, App sẽ chỉ gọi API tối đa 1 lần mỗi 2 phút, đảm bảo tài khoản không bao giờ bị khóa do Spam API.

### 2. Nâng cấp Giao diện Profile Card (UI/UX)
- **Mở rộng hiển thị Mức sử dụng (Usage Limits):** Khối hiển thị thanh trượt mức sử dụng theo phiên và tuần đã được thiết kế lại cấu trúc. Các thanh tiến trình giờ đây sẽ chiếm full-width ở nửa dưới của thẻ (Card), kéo dài đến tận mép phải và nằm phía dưới các nút thao tác. Giao diện trở nên liền mạch, dễ nhìn và căn chỉnh đồng bộ với các thông tin phía trên.
- **Tối ưu vị trí Nút Refresh:** Nút làm mới trạng thái usage được tinh chỉnh (`top-2`, bổ sung tính năng `cursor-pointer`) để hiển thị khớp hơn với khoảng trống phía trên của khung Progress.

### 3. Sửa lỗi logic phân loại Tài khoản (Account Type Logic)
- **Giải quyết triệt để lỗi phân loại "Tài khoản Cá nhân":** Trước đây, các tài khoản tổ chức (vd: MKT Abc) vẫn bị nhầm lẫn hiển thị là "Tài khoản Cá nhân" do trường `info.organizationUuid` thỉnh thoảng bị khuyết.
- **Giải pháp:** Cập nhật điều kiện lọc thông minh. Hệ thống hiện tại sẽ tham chiếu chéo thêm thông tin định danh trực tiếp từ Oauth Profile của người dùng (`profile.oauthAccount.organizationName` và `profile.oauthAccount.organizationUuid`), giúp dán nhãn "Tài khoản Tổ chức" đạt độ chuẩn xác 100%.

### 4. Tự động hóa Cập nhật (Auto-Updater Integration)
- **Chuẩn bị hạ tầng Auto-update:** Tích hợp `pubkey` và cấu hình endpoint tự động tải bản cập nhật mới từ GitHub Releases (`latest.json`) vào file config cốt lõi (`tauri.conf.json`). 
- **Phiên bản mới:** Đồng bộ toàn cục package version lên `1.0.3` để sẵn sàng cho bản release chính thức.

### 5. Fix Lỗi Build Core (Tauri Developer Fix)
- Khắc phục lỗi compile khi thao tác build Tauri (Rust) do cấu hình `process:allow-relaunch` không tồn tại.
- Hệ thống đã được map lại permission đúng chuẩn (`process:allow-restart`), đảm bảo khả năng build app từ source code một cách trơn tru.

---

*Bản cập nhật v1.0.3 mang lại "hương vị" của một ứng dụng quản lý chuyên nghiệp thực thụ: thông minh ngầm định, giao diện tối ưu không điểm mù và chuẩn xác đến từng metadata.*
