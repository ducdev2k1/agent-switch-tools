# Ghi chú phát hành Agent Switch Tools v1.0.18

**Agent Switch Tools v1.0.18** sửa một lỗi khó chịu tồn tại lâu trên **Linux (Wayland)**: sau khi app bị ẩn xuống tray rồi mở lại, các nút thu nhỏ/phóng to/đóng trên cửa sổ đôi khi không phản hồi khi bấm. Bản này cũng bổ sung tùy chọn "Khởi động ẩn xuống tray" cho ai muốn app tự chạy cùng hệ thống nhưng không muốn dashboard tự bật lên mỗi lần.

## Có gì mới?

### 1. Sửa lỗi các nút cửa sổ không phản hồi trên Linux/Wayland

Trên Wayland, các nút title bar do GTK vẽ có một lỗi đã biết: sau khi cửa sổ bị ẩn rồi hiện lại — đúng như khi bấm đóng app xuống tray rồi mở lại — nút đóng/thu nhỏ/phóng to có thể mất khả năng nhận click cho tới khi cửa sổ được resize.

- App giờ tự động "đánh thức" lại các nút điều khiển cửa sổ mỗi khi cửa sổ được focus lại, bằng cách bật/tắt nhanh thuộc tính resizable — tạo hiệu ứng giống resize thủ công nhưng không hề nhìn thấy được.
- Chỉ áp dụng trên Linux; hành vi trên macOS và Windows không thay đổi.
- Tham khảo: [tauri-apps/tauri#11856](https://github.com/tauri-apps/tauri/issues/11856), [tauri-apps/tauri#13440](https://github.com/tauri-apps/tauri/issues/13440).

### 2. Mới: Khởi động ẩn xuống tray

Một toggle mới trong **Cài đặt → Chung → Khởi động**, ngay cạnh "Khởi động cùng hệ thống":

- Khi bật, app khởi động chỉ hiện icon tray — dashboard không tự mở lên.
- Mở dashboard bất kỳ lúc nào từ menu tray ("Open Dashboard") hoặc mở lại app.
- **Mặc định tắt** — dashboard vẫn tự mở khi khởi động như trước, trừ khi bạn chủ động bật tùy chọn này.

---

## Lưu ý

- Fix nút cửa sổ chỉ áp dụng trên Linux; macOS và Windows không thay đổi hành vi.
- Cài đặt "Khởi động ẩn xuống tray" có hiệu lực từ lần mở app kế tiếp.

_v1.0.18 giúp các nút điều khiển cửa sổ hoạt động ổn định trên Linux/Wayland, và cho người dùng bật autostart thêm lựa chọn không cần dashboard tự bật lên ngay._
