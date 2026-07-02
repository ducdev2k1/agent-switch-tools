# Ghi chú phát hành Agent Switch Tools v1.0.15

**Agent Switch Tools v1.0.15** sửa lỗi nghiêm trọng khiến app không phát hiện được việc đăng nhập tài khoản khác bên ngoài app (có thể làm mất bản sao lưu tài khoản cũ), bổ sung cơ chế giữ bản sao lưu luôn "tươi", chuyển nhật ký hoạt động sang dạng bảng, và khắc phục hiện tượng nháy giao diện mỗi khi focus lại cửa sổ.

## Có gì mới?

### 1. Sửa lỗi không phát hiện đăng nhập ngoài app (quan trọng)

Từ v1.0.11, app đọc nhầm danh tính tài khoản từ cache của chính nó trong `~/.claude.json` thay vì trường `oauthAccount` do Claude Code ghi. Hệ quả khi bạn chạy `claude /login` với tài khoản khác:

- App **không nhận ra** tài khoản đã đổi — tài khoản cũ vẫn hiển thị là ACTIVE.
- Tài khoản mới **không được thêm** vào danh sách.
- Tệ nhất: bấm **"Lưu Hiện tại"** lúc đó sẽ **ghi đè bản sao lưu của tài khoản cũ bằng token của tài khoản mới** — mất token tài khoản cũ.

Bản này đọc đúng nguồn sự thật do Claude Code ghi:

- Đăng nhập ngoài app được phát hiện ngay lần mở/focus app kế tiếp: tài khoản mới tự được lưu thành profile (kèm đầy đủ danh tính), tài khoản cũ giữ nguyên, kèm thông báo trong app.
- Khi chuyển tài khoản trong app, danh tính tài khoản đích luôn được ghi lại chính xác — loại bỏ nguy cơ phát hiện nhầm rồi ghi đè sai thư mục.
- Thẻ tài khoản hiển thị lại **tên hiển thị** và **tên tổ chức** (bị mất từ v1.0.11).

> **Lưu ý:** nếu trước khi cập nhật bạn từng bấm "Lưu Hiện tại" sau khi đăng nhập tài khoản khác, bản sao lưu của tài khoản cũ có thể đã chứa token sai. Hãy `claude /login` lại tài khoản đó một lần — app sẽ tự lưu lại đúng.

### 2. Bản sao lưu luôn là mới nhất

Claude Code xoay vòng token liên tục trong lúc bạn dùng, nên bản sao lưu chụp từ lần chuyển tài khoản trước có thể đã cũ. Giờ đây **mỗi lần mở/focus app**, bản sao lưu và danh tính của tài khoản đang active được đồng bộ với trạng thái mới nhất — khi bạn đăng nhập tài khoản khác, snapshot giữ lại của tài khoản cũ luôn là bản "tươi" nhất có thể.

### 3. Nhật ký hoạt động dạng bảng

Tab **Phiên tự động**: nhật ký hoạt động chuyển từ text thô sang **bảng 4 cột** (Thời gian / Tài khoản / Trạng thái / Chi tiết):

- Dòng mới nhất lên đầu, tiêu đề cột dính khi cuộn.
- Ngày giờ định dạng **dd/mm/yyyy hh:mm** — kể cả các mốc reset trong phần chi tiết, tự quy về múi giờ máy bạn (hết cảnh đọc `2026-06-30T11:09:59+00:00`).
- Trạng thái hiển thị bằng badge màu (thành công / tạm hoãn / thất bại / bỏ qua).

### 4. Hết nháy giao diện khi focus lại app

- Trước đây mỗi lần focus lại cửa sổ, danh sách tài khoản bị thay bằng skeleton một nhịp rồi vẽ lại — phần thông tin tài khoản và quota "nháy" một phát. Giờ dữ liệu cũ giữ nguyên trên màn hình và chỉ cập nhật êm phía sau.
- Quota tự làm mới khi focus, **giới hạn 2 phút/lần** (khớp cache 2 phút phía backend) — chuyển qua lại giữa các cửa sổ trong 2 phút không phát sinh request nào tới API Anthropic, không lo rate limit.

---

## Lưu ý

- Toàn bộ phát hiện/lưu tài khoản chạy khi bạn mở hoặc focus cửa sổ app (không chạy ngầm trong tray) — sau khi `claude /login` tài khoản khác, mở app lên một lần để app ghi nhận.
- Nếu app không chạy giữa hai lần đăng nhập ngoài liên tiếp, tài khoản ở giữa không thể được lưu (app chưa từng thấy token của nó).

_v1.0.15 tập trung bảo toàn dữ liệu tài khoản khi đăng nhập ngoài app và làm giao diện mượt hơn._
