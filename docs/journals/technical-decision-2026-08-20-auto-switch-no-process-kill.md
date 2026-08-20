# Auto Switch Rule không can thiệp vào Claude Code đang chạy

**Ngày**: 2026-08-20 (sửa lại cùng ngày sau khi kiểm chứng thực tế)
**Thành phần**: Auto Switch Rule, credential switching, quota worker
**Trạng thái**: Accepted
**Commit**: (chưa commit)

## Quyết Định

Khi Auto Switch Rule kích hoạt, ứng dụng **chỉ ghi credential của Fallback Profile** và không can thiệp vào process `claude` đang chạy — không kill, không restart, không chờ nó tắt.

## Hành Vi Thực Tế Của Claude Code

Điểm này ban đầu bị hiểu sai, cần ghi lại cho rõ vì nó chi phối toàn bộ thiết kế và mọi câu thông báo cho người dùng.

Ứng dụng ghi credential vào `~/.claude/.credentials.json` — đúng file mà Claude Code CLI dùng. CLI đọc file này **theo từng request** (nó cũng tự ghi lại file đó khi refresh OAuth token, nên file là state dùng chung, không phải bản copy nạp một lần lúc khởi động).

Hệ quả, kiểm chứng bằng quan sát trực tiếp trên máy thật:

- **Quota**: phiên đang chạy dùng credential mới **ngay**. Tài khoản mới bị trừ quota đúng, không cần restart gì.
- **Thông tin hiển thị**: phiên đang chạy vẫn hiện tài khoản **cũ**. Phần này được CLI cache lúc mở phiên. Mở phiên mới thì hiển thị đúng.

Nói cách khác, việc đổi có hiệu lực tức thì ở chỗ quan trọng (ai bị trừ quota) và trễ ở chỗ không quan trọng (nhãn hiển thị).

## Sai Sót Ban Đầu

Bản đầu của ADR này khẳng định phiên đang chạy **không** nhận credential mới và người dùng **phải** restart Claude Code. Nguồn của khẳng định đó là một chuỗi có sẵn trong `config_commands.rs` (*"Switched credentials. Restart Claude Code to use new account."*) được coi là đúng mà không kiểm chứng.

Nó sai, và cái sai đó đã lan ra 5 chỗ: callout trong tab Auto Switch, một biến thể toast riêng, phần thân desktop notification, mục tính năng trong docs, và ADR này. Chuỗi gốc trong `config_commands.rs` cũng đã được sửa cùng lúc — nó hiện trên mọi lần switch tay, không riêng auto-switch.

Bài học: một chuỗi text có sẵn trong repo không phải bằng chứng về hành vi của hệ thống bên ngoài. Nó chỉ là điều một người nào đó từng tin.

## Các Lựa Chọn Đã Cân Nhắc

1. **Ghi credential rồi để process yên** — đã chọn. Việc đổi có hiệu lực ngay về mặt quota, nên không có gì phải bù đắp.
2. **Kill rồi khởi động lại `claude`.** Loại. Trước đây phương án này bị loại vì rủi ro phá công việc đang dở; giờ nó còn **vô nghĩa** nữa, vì credential đã có hiệu lực mà không cần restart. `check_claude_running()` cũng chỉ là `pgrep -f claude` — không phân biệt được phiên nào đang làm gì.
3. **Hoãn switch tới khi process tắt.** Loại: làm mất đúng cái giá trị mà tính năng đem lại, để đổi lấy một nhãn hiển thị đúng sớm hơn.

## Hệ Quả

- Auto Switch Rule có hiệu lực **tức thì** về quota, kể cả khi người dùng đang làm việc giữa phiên.
- Vì vậy thông báo càng quan trọng: tài khoản bị trừ quota đã đổi mà người dùng không làm gì, và phiên đang chạy vẫn hiện tên tài khoản cũ nên bản thân phiên đó không nói cho họ biết. Desktop notification + toast là kênh duy nhất cho họ biết chuyện gì vừa xảy ra.
- Mọi câu chữ hướng người dùng đi restart Claude Code đều là thông tin sai — trừ khi mục đích là để nhãn hiển thị khớp lại, và khi đó phải nói đúng lý do đó.
- `claude_was_running` từ `SwitchResult` không còn được Auto Switch Rule dùng để phân nhánh thông báo.
