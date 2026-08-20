# Ghi chú phát hành Agent Switch Tools v1.1.0

**Agent Switch Tools v1.1.0** thêm **Auto Switch Rule** — app tự chuyển sang tài khoản còn nhiều hạn mức nhất khi tài khoản bạn đang dùng chạm ngưỡng bạn đặt. Trước đây bạn phải tự để ý con số phần trăm rồi tự bấm chuyển; giờ việc đó chạy nền.

## Có gì mới?

### 1. Tab "Auto Switch" trong Cài đặt

Mặc định **tắt**. Bật lên rồi cấu hình 2 giá trị:

- **Ngưỡng chuyển (Switch Threshold)** — 50–99%, mặc định 90%. Khi tài khoản đang dùng chạm mức này, app chuyển sang tài khoản khác.
- **Khoảng chờ (Cooldown)** — 5–120 phút, mặc định 5. Giới hạn tần suất chuyển tự động.

Ngưỡng được đo trên **hạn mức 5 giờ** của Claude Code — hạn mức thực sự chặn công việc. Hạn mức theo tuần (7 ngày, 7 ngày Sonnet) được bỏ qua có chủ đích: một tài khoản đã cạn cả tuần thì chuyển đi trong năm phút cũng không cứu được, việc đó cần bạn xử lý bằng cách khác.

### 2. Chọn tài khoản đích: còn nhiều hạn mức nhất

App vốn đã cập nhật hạn mức của **mọi** tài khoản mỗi 5 phút, nên nó biết sẵn tài khoản nào rảnh nhất mà không cần gọi thêm API nào. Khi rule kích hoạt, nó chuyển sang tài khoản có mức sử dụng 5 giờ thấp nhất trong số các tài khoản còn dưới ngưỡng.

Sau khi chuyển, app **ở lại** tài khoản mới — không tự nhảy về tài khoản cũ khi hạn mức của tài khoản đó reset. Bạn tự chuyển về khi muốn.

### 3. Luôn được thông báo, kể cả khi app ở tray

Vì rule chạy nền, việc thông báo là bắt buộc chứ không phải tuỳ chọn:

- **Thông báo hệ thống** — thấy được ngay cả khi app đang thu nhỏ xuống tray.
- **Toast trong app** — hiện ở bất kỳ trang nào, không cần đang mở tab Auto Switch.
- **Menu tray** được cập nhật để hiển thị tài khoản mới.

Nếu Claude Code đang chạy lúc chuyển, thông báo sẽ nhắc **khởi động lại Claude Code**. Đây là điều quan trọng cần biết: việc chuyển chỉ ghi lại credential, còn một phiên `claude` đang chạy vẫn dùng credential cũ cho tới khi bạn khởi động lại nó. App **không** tự tắt Claude Code của bạn — làm vậy có thể phá công việc đang dở.

### 4. Khi mọi tài khoản đều đã cạn

Không chuyển gì cả, và chỉ thông báo **một lần** thay vì nhắc lại mỗi vài phút. Cờ này tự đặt lại ngay khi có tài khoản tụt xuống dưới ngưỡng.

### 5. Lịch sử chuyển tự động

Tab Auto Switch lưu lại mọi lần chuyển: thời điểm, tài khoản rời đi, tài khoản được chuyển tới, và mức sử dụng đã kích hoạt việc chuyển. Dùng để truy lại khi bạn thấy tài khoản bị đổi mà không rõ vì sao.

---

## Lưu ý

- Chỉ áp dụng cho **Claude Code**. Cursor, Windsurf và Antigravity chưa được hỗ trợ trong bản này.
- Cần tối thiểu **2 tài khoản đã lưu** để rule có thể chuyển. Nếu chỉ có một tài khoản, app sẽ chỉ thông báo khi nó cạn.
- Vì hạn mức được cập nhật mỗi 5 phút, việc chuyển có thể diễn ra chậm tối đa 5 phút sau khi bạn thực sự vượt ngưỡng.
- Khoảng chờ được lưu xuống đĩa, nên tắt rồi mở lại app không làm nó về 0.

_v1.1.0 biến việc theo dõi hạn mức thủ công thành việc app tự làm, và luôn nói cho bạn biết nó đã làm gì._
