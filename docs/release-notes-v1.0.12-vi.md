# Ghi chú phát hành Agent Switch Tools v1.0.12

**Agent Switch Tools v1.0.12** hỗ trợ đầy đủ đa biến thể cho Antigravity — Desktop, IDE và CLI được quản lý độc lập — đồng thời cập nhật engine quota theo chính sách rate-limit mới của Antigravity (**Weekly + 5 giờ**). Chuyển tab giờ tức thì (không reload nhấp nháy), và credentials OAuth được nạp tự động khi build.

## Có gì mới?

### 1. Antigravity: Desktop, IDE & CLI — hỗ trợ tất cả

Google đã tách Antigravity thành 3 sản phẩm riêng, mỗi cái lưu credentials khác nhau. App giờ nhận diện và quản lý cả ba:

| Biến thể | Nguồn |
|----------|-------|
| **Antigravity** (Desktop) | `~/.config/Antigravity` → `state.vscdb` |
| **Antigravity IDE** | `~/.config/Antigravity IDE` → `state.vscdb` (OAuth proto, không còn `antigravityAuthStatus`) |
| **Antigravity CLI** | `~/.gemini/antigravity-cli/antigravity-oauth-token` (file JSON) |

Một abstraction `CredentialSource` mới giúp cùng một luồng profile/switch/quota chạy được cho cả store `state.vscdb` (VS Code) lẫn file token JSON của CLI.

### 2. Gộp 1 tab với các sub-tab biến thể

Ba biến thể Antigravity giờ nằm chung trong **một tab "Antigravity"** trên thanh tab, bên trong có **sub-tab Desktop / IDE / CLI** — thay vì để 3 icon gần giống nhau gây rối thanh tab.

### 3. Mô hình quota mới — giới hạn Weekly + 5 giờ

Theo cập nhật chính sách Gemini của Google, Antigravity giờ tính usage theo **giới hạn Weekly và 5 giờ cho từng nhóm model**. Dashboard giờ đọc endpoint mới `retrieveUserQuotaSummary` (đúng cái lệnh `usage` native dùng) và hiển thị:

- **Gemini — Weekly Limit / Five Hour Limit**
- **Claude and GPT — Weekly Limit / Five Hour Limit**

Thay cho `remainingFraction` 1 cửa sổ theo từng model trước đây, và áp dụng cho cả 3 biến thể.

### 4. Thông tin tài khoản cho IDE & CLI

Các bản Antigravity mới không còn lưu email tài khoản ở local. App lấy email qua endpoint `userinfo` của Google (dùng chính token OAuth của tài khoản) và cache vào profile đã lưu — nên email và avatar hiển thị đúng trên cả Desktop, IDE và CLI.

### 5. Chuyển tab tức thì — không reload, không nhảy layout

Chuyển tab không còn fetch lại từ đầu (vốn gây nhấp nháy skeleton và xô lệch layout). Profiles và quota giờ được cache trong bộ nhớ và hiển thị ngay, refresh ngầm phía sau.

### 6. Parser token OAuth bền hơn

Parser protobuf của OAuth-token Antigravity giờ tìm token theo sentinel key thay vì theo vị trí, nên vẫn chạy đúng kể cả khi bản mới đảo thứ tự field trong blob.

### 7. Credentials OAuth khi build

Credentials client OAuth giờ được nạp tự động từ file `.env` local lúc build (qua `build.rs`), còn CI vẫn inject từ secrets. Hết lỗi refresh do client rỗng ở bản build local.

---

## Lưu ý

- **Quota Antigravity CLI cần tài khoản đủ điều kiện.** Nếu Google báo *"Verify your account to continue"* (ví dụ tài khoản chưa xác minh SĐT), API quota sẽ không trả dữ liệu — đúng với cả CLI native lẫn app này — cho tới khi tài khoản được xác minh.
- Cursor và Windsurf tạm thời được ẩn khỏi dashboard.

_v1.0.12 biến mọi phiên bản Antigravity — Desktop, IDE và CLI — thành tài khoản hạng nhất, quan sát được đầy đủ, khớp với chính sách quota mới nhất của Google._
