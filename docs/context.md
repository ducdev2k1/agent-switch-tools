# Agent Switch Tools

Ứng dụng desktop offline quản lý nhiều tài khoản AI coding agent (Claude Code, Cursor, Windsurf, Antigravity): lưu credential thành nhiều bộ, đổi qua lại giữa chúng, và theo dõi mức tiêu thụ quota của từng bộ.

Tài liệu này là **glossary** — chỉ định nghĩa từ vựng của domain. Không chứa chi tiết triển khai.

## Định danh & credential

**Profile**:
Một bộ credential đã lưu của một tài khoản, được định danh bằng email của tài khoản đó.
_Avoid_: Account, tài khoản (chỉ dùng ở bản dịch UI), user

**Active Profile**:
Profile mà agent CLI/IDE đang thực sự dùng ở thời điểm hiện tại. Chỉ có tối đa một Active Profile cho mỗi agent.
_Avoid_: Current account, selected profile

**Credential Source**:
Nơi credential của Active Profile được lưu trên máy — có thể là file, hoặc keystore của hệ điều hành.
_Avoid_: Credential store, vault

**Drift**:
Tình trạng Active Profile trên máy đã bị thay đổi bởi thao tác bên ngoài ứng dụng (ví dụ user tự đăng nhập lại bằng CLI), khiến trạng thái ứng dụng ghi nhận không còn khớp thực tế.
_Avoid_: Desync, mismatch, conflict

## Quota

**Usage Bucket**:
Một hạn mức tiêu thụ có chu kỳ reset riêng, kèm mốc thời gian reset. Mỗi provider công bố một tập bucket khác nhau.
_Avoid_: Limit, quota slot, window

**Utilization**:
Mức tiêu thụ của một Usage Bucket, tính theo phần trăm. Provider có thể công bố theo hệ "đã dùng" hoặc hệ "còn lại"; hai hệ này ngược chiều nhau nên luôn phải biết đang đọc hệ nào.
_Avoid_: Usage percent, consumption

**Five-Hour Window**:
Usage Bucket ngắn nhất của Claude Code — hạn mức reset theo chu kỳ 5 giờ. Đây là bucket chặn công việc thực tế của người dùng, khác với các bucket theo tuần.
_Avoid_: Short window, session limit

## Tự động hoá

**Auto Switch Rule**:
Quy tắc do người dùng cấu hình, cho phép ứng dụng tự đổi Active Profile khi profile đang dùng cạn quota.
_Avoid_: Auto switch feature, quota guard, failover

**Switch Threshold**:
Mức Utilization mà khi Active Profile chạm tới, Auto Switch Rule được kích hoạt.
_Avoid_: Limit, cutoff, max usage

**Fallback Profile**:
Profile được Auto Switch Rule chọn làm đích để chuyển sang. Là profile còn nhiều quota nhất trong số các profile hợp lệ.
_Avoid_: Backup account, next profile, target

**Cooldown**:
Khoảng thời gian tối thiểu giữa hai lần Auto Switch Rule được phép đổi profile, tính từ lần đổi tự động gần nhất.
_Avoid_: Debounce, delay, throttle

**Auto Prime**:
Quy tắc tách biệt với Auto Switch Rule: chủ động mở chu kỳ quota của một profile vào một giờ cố định hằng ngày, để chu kỳ reset về sau khớp với nhịp làm việc của người dùng.
_Avoid_: Warm-up, scheduled session, keep-alive
