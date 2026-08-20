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

Điều này quan trọng hơn nghe tưởng. Việc chuyển ghi lại `~/.claude/.credentials.json` — file mà Claude Code đọc theo từng request — nên phiên bạn đang làm giữa dở **bị trừ quota sang tài khoản mới ngay lập tức**, trong khi tên tài khoản nó *hiển thị* vẫn là tài khoản cũ cho tới khi bạn mở phiên mới. Nghĩa là chính phiên đó không nói cho bạn biết chuyện gì vừa xảy ra; chỉ thông báo mới làm được. App **không** tự tắt Claude Code của bạn: việc đó vừa không cần thiết (credential đã có hiệu lực) vừa có thể phá công việc đang dở.

### 4. Khi mọi tài khoản đều đã cạn

Không chuyển gì cả, và chỉ thông báo **một lần** thay vì nhắc lại mỗi vài phút. Cờ này tự đặt lại ngay khi có tài khoản tụt xuống dưới ngưỡng.

### 5. Lịch sử chuyển tự động

Tab Auto Switch lưu lại mọi lần chuyển: thời điểm, tài khoản rời đi, tài khoản được chuyển tới, và mức sử dụng đã kích hoạt việc chuyển. Dùng để truy lại khi bạn thấy tài khoản bị đổi mà không rõ vì sao.

### 6. Tab Phiên tự động không còn lag

Không thuộc tính năng tự động đổi, nhưng cùng gốc rễ nên sửa luôn.

Activity log của Phiên tự động trước đây được gửi sang giao diện dưới dạng **một chuỗi duy nhất**, rồi render **toàn bộ** số dòng cùng lúc. Trên máy có log đã phình tới 25.000 dòng, mỗi lần bấm vào tab là chuyển 2,5 MB qua và dựng khoảng 125.000 ô bảng — mất vài giây mới hiện.

Nguyên nhân log phình: một profile đã bị xóa thông tin đăng nhập vẫn bị scheduler ghi `skip | credentials not found` **mỗi phút, suốt nhiều tuần**. Gần như toàn bộ 25.000 dòng đó là rác.

Đã sửa cả ba tầng:

- Scheduler bỏ qua im lặng profile không còn thông tin đăng nhập, vẫn giữ lịch prime của nó.
- Activity log tự giới hạn: vượt 5.000 dòng thì ghi lại còn 2.000 dòng mới nhất. Log đang quá cỡ được cắt ngay lần đầu mở bản này.
- Bảng log được phân trang 100 dòng, dữ liệu được parse ở phía ứng dụng trước khi tới giao diện. Bảng lịch sử của Tự động đổi dùng cùng cách.

---

## Lưu ý

- Chỉ áp dụng cho **Claude Code**. Cursor, Windsurf và Antigravity chưa được hỗ trợ trong bản này.
- Cần tối thiểu **2 tài khoản đã lưu** để rule có thể chuyển. Nếu chỉ có một tài khoản, app sẽ chỉ thông báo khi nó cạn.
- Vì hạn mức được cập nhật mỗi 5 phút, việc chuyển có thể diễn ra chậm tối đa 5 phút sau khi bạn thực sự vượt ngưỡng.
- Không cần khởi động lại để việc chuyển có hiệu lực. Chỉ khởi động lại nếu bạn muốn tên tài khoản hiển thị trong phiên đang chạy khớp lại.
- Khoảng chờ được lưu xuống đĩa, nên tắt rồi mở lại app không làm nó về 0.

_v1.1.0 biến việc theo dõi hạn mức thủ công thành việc app tự làm, và luôn nói cho bạn biết nó đã làm gì._
