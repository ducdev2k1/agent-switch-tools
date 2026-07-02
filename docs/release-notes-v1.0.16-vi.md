# Ghi chú phát hành Agent Switch Tools v1.0.16

**Agent Switch Tools v1.0.16** tập trung làm trải nghiệm chuyển tài khoản nhanh và mượt hơn: bỏ hộp thoại xác nhận khi switch, chuyển nhanh từ tray chạy hoàn toàn ngầm, danh sách giữ thứ tự ổn định, hoàn thiện Việt hóa cho chế độ xem danh sách, và tinh gọn giao diện (bỏ viền xanh đậm, cửa sổ mặc định rộng hơn).

## Có gì mới?

### 1. Chuyển tài khoản một chạm — bỏ hộp thoại xác nhận

Trước đây mỗi lần chuyển tài khoản (Claude Code hoặc IDE) app đều bật hộp thoại xác nhận. Giờ **bấm vào tài khoản là chuyển ngay**:

- Áp dụng cho cả thẻ (grid), bảng (list) và menu tray.
- Các cảnh báo quan trọng vẫn còn, chỉ chuyển sang dạng thông báo sau khi switch: nếu Claude Code đang chạy, app nhắc khởi động lại để dùng tài khoản mới.
- Với IDE: nếu IDE đang chạy lúc chuyển, app **tự khởi động lại IDE** như trước — chỉ bỏ bước hỏi.

### 2. Chuyển nhanh từ System Tray chạy hoàn toàn ngầm

Trước đây bấm profile trên tray sẽ mở cửa sổ app rồi mới xử lý — và với IDE thì chỉ mở đúng tab, bạn vẫn phải bấm chọn lại. Giờ đây:

- Bấm profile trên tray → **switch ngay trong nền**, không bật cửa sổ.
- Hoạt động kể cả khi dashboard **chưa từng mở** — tray không còn phụ thuộc giao diện.
- Nếu app đang mở, danh sách tự làm mới và hiện thông báo "Đã chuyển sang...".
- Tài khoản IDE: chuyển xong tự khởi động lại IDE nếu nó đang chạy.

### 3. Danh sách tài khoản giữ thứ tự ổn định

Trước đây sau khi chuyển tài khoản, tài khoản active bị đẩy lên đầu danh sách — vị trí các thẻ nhảy lung tung, khó tìm lại tài khoản vừa thao tác. Giờ danh sách **luôn xếp theo alphabet**, chuyển tài khoản không làm thay đổi vị trí; tài khoản active nhận biết qua nền xanh nhạt và badge HOẠT ĐỘNG.

### 4. Việt hóa đầy đủ chế độ xem danh sách

Chế độ xem danh sách (list view) trước đây hiện mã key thô ở tiêu đề cột. Giờ đã dịch đầy đủ theo ngôn ngữ app:

- Tiêu đề cột: **Email / Gói thành viên / Quota mô hình / Hết hạn lúc / Trạng thái / Thao tác**.
- Các nút thao tác (làm mới token, làm mới quota, chuyển tài khoản, xóa) có tooltip khi rê chuột.

### 5. Giao diện tinh gọn hơn

- **Bỏ dải viền xanh đậm** bên trái thẻ tài khoản active — bớt chói, vẫn phân biệt được qua nền xanh nhạt, chấm xanh và badge HOẠT ĐỘNG.
- **Cửa sổ mặc định rộng hơn**: 1200×720 (trước là 900×640) — lưới tài khoản hiện đủ **3 thẻ mỗi hàng** ngay khi mở app, không phải kéo giãn thủ công.

---

## Lưu ý

- Kích thước cửa sổ mới chỉ áp dụng cho cửa sổ mở mới; nếu hệ điều hành đang nhớ kích thước cũ, kéo giãn một lần là xong.
- Vì không còn bước xác nhận, hãy để ý thông báo sau khi switch — nếu Claude Code đang chạy, cần khởi động lại nó để dùng tài khoản mới.

_v1.0.16 tập trung rút ngắn thao tác chuyển tài khoản xuống một chạm và làm giao diện gọn gàng, ổn định hơn._
