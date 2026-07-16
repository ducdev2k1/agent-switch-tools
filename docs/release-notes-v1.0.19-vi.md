# Ghi chú phát hành Agent Switch Tools v1.0.19

**Agent Switch Tools v1.0.19** sửa lỗi chuyển tài khoản trên **macOS** với các profile được import từ máy khác — trường hợp zip folder tài khoản ở máy này, copy vào thư mục profiles trên máy Mac, rồi chuyển sang nó thì báo **"Failed to write credential to macOS Keychain"** (đôi khi kèm popup hỏi mật khẩu Keychain). Profile tự đăng nhập trên chính máy đó lại chuyển bình thường, khiến lỗi này đặc biệt khó hiểu.

## Có gì mới?

### 1. Sửa: chuyển sang profile được chia sẻ/import trên macOS

Có hai vấn đề độc lập chồng lên nhau trên luồng này, và cả hai đều đã được sửa:

- **Ký tự xuống dòng cuối file làm hỏng bước xác nhận ghi.** File `credentials.json` copy từ máy khác thường có ký tự xuống dòng ở cuối. Credential thực ra đã được ghi vào Keychain thành công, nhưng macOS trả về giá trị chứa ký tự không in được dưới dạng mã hex khi đọc lại, nên bước xác nhận của app không bao giờ khớp và việc chuyển bị báo thất bại. Giờ credential được lưu sau khi loại bỏ khoảng trắng thừa, nên vòng ghi/đọc luôn khớp.
- **Mục Keychain do app khác tạo chặn cập nhật im lặng.** Nếu mục Keychain do app khác tạo (ví dụ Claude Code CLI khi đăng nhập lần đầu), danh sách quyền truy cập của nó có thể từ chối việc cập nhật tại chỗ và kích hoạt popup hỏi mật khẩu macOS. App giờ tự xóa và tạo lại mục đó với quyền truy cập của chính app, nên các lần chuyển sau đều im lặng.

### 2. Thao tác Keychain không thể làm treo app nữa

Trước đây, nếu macOS giữ một lệnh Keychain lại sau hộp thoại bảo mật, app sẽ chờ vô hạn. Giờ:

- Mọi lệnh `security` được giới hạn **15 giây**.
- Lệnh bị kẹt trả về lỗi rõ ràng nêu đúng lệnh con và nguyên nhân khả nghi (ví dụ "timed out — likely blocked on a keychain dialog") thay vì làm đứng việc chuyển tài khoản.
- Thông báo lỗi Keychain hiển thị nguyên nhân thật từ macOS (keychain bị khóa, bị từ chối truy cập, hết thời gian chờ) thay vì một dòng lỗi chung chung.

### 3. Test Keychain thật trong CI

Đúng các kịch bản lỗi ở trên giờ được kiểm tra bằng test tích hợp chạy trên Keychain thật của macOS, trên cả runner Intel lẫn Apple Silicon của GitHub Actions, để việc chuyển tài khoản không tái phát lỗi.

---

## Lưu ý

- Mọi thay đổi chỉ áp dụng trên macOS; Linux và Windows vẫn dùng file credentials y như trước.
- Nếu việc chuyển vẫn thất bại trên máy Mac của bạn, thông báo lỗi giờ đã nêu nguyên nhân thật — hãy kèm nó khi báo lỗi.

_v1.0.19 giúp chuyển sang profile import hoạt động ổn định trên macOS và đảm bảo app không bao giờ bị treo vì hộp thoại Keychain._
