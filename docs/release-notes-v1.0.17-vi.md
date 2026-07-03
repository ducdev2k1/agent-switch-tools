# Ghi chú phát hành Agent Switch Tools v1.0.17

**Agent Switch Tools v1.0.17** sửa một vấn đề nền tảng trên **macOS**: từ Claude Code 2.x, credentials của Claude Code được lưu trong **macOS Keychain** chứ không còn nằm ở file `~/.claude/.credentials.json`. Các bản trước chỉ đọc/ghi file nên trên macOS việc chuyển tài khoản không thực sự có hiệu lực. Bản này đưa toàn bộ luồng credentials của tài khoản active đi qua Keychain trên macOS, trong khi Linux/Windows giữ nguyên như cũ.

## Có gì mới?

### 1. Hỗ trợ macOS Keychain cho tài khoản active

Trên macOS, app giờ đọc và ghi credentials của tài khoản **đang active** trực tiếp trong login Keychain — đúng nơi Claude Code CLI thực sự đọc:

- Dùng đúng tên keychain của Claude Code 2.x: `Claude Code-credentials-<hash thư mục cấu hình>`, có fallback về tên cũ và về file.
- Ghi an toàn: cập nhật tại chỗ, **đọc lại để xác nhận** đã ghi đúng chỗ, và **thử lại tối đa 3 lần** nếu Keychain tạm khóa.
- **Đồng bộ Keychain ↔ file**: nếu file credentials đang tồn tại, app cũng ghi bản sao ra file để tác vụ nền vẫn đọc được khi Keychain bị khóa — nhưng không tự tạo file mới nếu chưa có.

### 2. Các thao tác đều nhận đúng tài khoản trên macOS

Sau bản này, trên macOS các chức năng sau đọc/ghi đúng đối tượng thay vì trượt vào một file không tồn tại:

- Hiển thị và nhận diện tài khoản **active** trong danh sách và trên tray.
- **Lưu** tài khoản hiện tại thành profile.
- **Chuyển** tài khoản (switch) — credentials mới thực sự được Claude Code sử dụng sau khi khởi động lại.
- **Đồng bộ trạng thái** (reconcile) khi bạn đăng nhập bằng `claude /login` bên ngoài app.
- **Làm mới token**, đọc **quota**, **priming**, và dữ liệu gửi qua **webhook**.

### 3. Linux & Windows: không thay đổi

Trên Linux và Windows, app tiếp tục dùng file `~/.claude/.credentials.json` y như trước — hành vi không đổi. Các profile đã lưu (trong `~/.agent-switch-tools/`) vẫn là file thường trên mọi hệ điều hành.

---

## Lưu ý

- Trên macOS, lần đầu ghi vào Keychain hệ điều hành có thể hỏi quyền truy cập Keychain — đây là hành vi bình thường.
- Nếu trước đây bạn dùng bản cũ trên macOS và thấy tài khoản active không hiện ra, sau khi cập nhật app sẽ đọc được từ Keychain.
- Sau khi switch, nếu Claude Code đang chạy, hãy khởi động lại nó để dùng tài khoản mới.

_v1.0.17 tập trung làm cho việc chuyển tài khoản Claude Code hoạt động đúng trên macOS bằng cách lưu credentials nơi Claude Code thực sự đọc — login Keychain._
