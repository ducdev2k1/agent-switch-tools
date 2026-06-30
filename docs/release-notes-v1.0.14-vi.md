# Ghi chú phát hành Agent Switch Tools v1.0.14

**Agent Switch Tools v1.0.14** sửa lỗi nghiêm trọng khiến việc làm mới token luôn thất bại, cải thiện phản hồi của nút làm mới token, và bổ sung **trình xem changelog ngay trong app** để bạn biết mỗi bản cập nhật có gì mới.

## Có gì mới?

### 1. Sửa lỗi làm mới token (HTTP 404)

Endpoint OAuth của Anthropic đã được dời, khiến đường dẫn cũ trả về `404 Not Found`. Hệ quả là **mọi lần làm mới token đều thất bại** — cả khi bấm nút thủ công lẫn cơ chế tự động chạy nền khi cập nhật quota.

Bản này trỏ sang endpoint còn hoạt động (`https://claude.ai/v1/oauth/token`):

- Nút làm mới token thủ công hoạt động trở lại.
- **Tự động làm mới token được khôi phục** — app lại tự giữ access token "tươi" khi lấy usage, qua worker nền (mỗi 5 phút) và khi priming.

### 2. Phản hồi rõ ràng khi làm mới token

- Khi làm mới **thất bại**, app hiện **thông báo lỗi cụ thể** (ví dụ token đã bị thu hồi) thay vì im lặng không phản hồi như trước.
- Nút làm mới token (🔑) giờ **luôn hiển thị** trên tài khoản đã hết hạn — không cần rê chuột vào thẻ mới thấy.

### 3. Trình xem changelog trong app

- Thêm mục **"Có gì mới"** trong tab **Cài đặt → Giới thiệu**: xem nhanh các thay đổi nổi bật của những bản gần đây, hoạt động hoàn toàn offline.
- Khi có bản cập nhật, **hộp thoại cập nhật** giờ hiển thị release notes của phiên bản mới ngay trước khi cài.

---

## Lưu ý

- Nếu sau khi cập nhật, làm mới token vẫn báo `400 invalid_grant` (không phải `404`), nghĩa là refresh token của tài khoản đó đã hết hiệu lực thật — hãy đăng nhập lại tài khoản đó. Cơ chế làm mới đã đúng.
- Việc làm mới token cho **tài khoản đã lưu (chưa active)** ghi đúng vào kho riêng của tài khoản đó, không ảnh hưởng tài khoản đang dùng.

_v1.0.14 tập trung khôi phục việc làm mới token và giúp bạn nắm rõ mỗi bản cập nhật có gì._
