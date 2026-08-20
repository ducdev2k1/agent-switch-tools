# Auto Switch Rule không kill Claude Code đang chạy

**Ngày**: 2026-08-20
**Thành phần**: Auto Switch Rule, credential switching, quota worker
**Trạng thái**: Accepted (chốt trước khi triển khai)
**Commit**: (chưa commit)

## Quyết Định

Khi Auto Switch Rule kích hoạt mà process `claude` đang chạy, ứng dụng **vẫn ghi credential của Fallback Profile và không can thiệp vào process đang chạy**. Session Claude Code hiện tại tiếp tục dùng credential cũ cho tới khi user tự restart; ứng dụng chỉ thông báo rằng đã đổi và cần restart để có hiệu lực.

## Bối Cảnh

Việc đổi Active Profile chỉ là ghi lại credential vào Credential Source. Process `claude` đọc credential lúc khởi động, nên **một session đang chạy không nhận credential mới** — đây là sự thật của tầng dưới, không phải lựa chọn thiết kế (`src-tauri/src/commands/config_commands.rs`, nhánh `claude_was_running`).

Với switch thủ công, hạn chế này chấp nhận được: user chủ động bấm nên biết mình vừa làm gì. Với switch **tự động** thì khác — nó xảy ra trong lúc user không nhìn app, thậm chí app đang ở tray.

## Các Lựa Chọn Đã Cân Nhắc

1. **Vẫn switch, chỉ thông báo** — đã chọn.
2. **Hoãn switch, chỉ cảnh báo, đợi user bấm xác nhận.** Loại: biến "tự động" thành "nhắc thủ công", làm mất gần hết giá trị của tính năng. Người dùng bật rule chính là để không phải tự theo dõi.
3. **Tự kill process `claude` rồi khởi động lại.** Loại: `check_claude_running()` chỉ là `pgrep -f claude` — nó không phân biệt được session nào, cũng không biết session đó đang làm gì. Kill có thể phá công việc đang dở của user. Cái giá của một lần kill sai lớn hơn nhiều so với lợi ích "credential có hiệu lực ngay".

## Hệ Quả

- Auto Switch Rule là cơ chế **eventual**: đảm bảo lần chạy Claude Code *tiếp theo* dùng profile còn quota, không đảm bảo session *hiện tại*.
- Vì hiệu lực bị hoãn tới lần restart, thông báo là phần bắt buộc của tính năng, không phải tuỳ chọn trang trí — nếu user không thấy thông báo thì họ sẽ tiếp tục làm việc trên profile đã cạn và không hiểu vì sao.
- Đây là lý do desktop notification được thêm vào (app có autostart + start-minimized, nên chỉ toast trong app là không đủ).
- Nếu về sau muốn hiệu lực tức thì, hướng đúng **không phải** kill process, mà là chờ Claude Code hỗ trợ nạp lại credential khi đang chạy.
