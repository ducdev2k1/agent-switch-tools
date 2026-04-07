# Claude Tools v1.0.7 - Ghi chú phát hành

**Claude Tools v1.0.7** bổ sung tính năng làm mới OAuth token một chạm và sửa lỗi cache khi chuyển tài khoản.

## Có gì mới?

### 1. Làm mới OAuth Token

- **Một chạm làm mới**: Tài khoản hết hạn hiển thị nút "Làm mới Token" cạnh badge "Hết hạn". Nhấn để refresh token — không cần đăng nhập lại.
- **Áp dụng cho mọi tài khoản**: Cả tài khoản active và saved (inactive) đều refresh được từ dashboard.
- **Dùng CLI để refresh**: Sử dụng `claude -p` để kích hoạt cùng luồng refresh như CLI chính thức. Với saved profiles, credentials được swap tạm, refresh xong rồi restore lại.

### 2. Sửa lỗi cache khi chuyển tài khoản

- **Không còn dữ liệu cũ**: Chuyển tài khoản trước đây hiển thị usage từ tài khoản cũ. Hook `useProfileUsage` giờ phát hiện thay đổi `isActive`, xóa dữ liệu cũ và force-refresh từ API.
- **Khôi phục Force Refresh**: Sửa lỗi nút refresh không truyền `forceRefresh` xuống backend, khiến luôn trả cache trong 2 phút TTL.

### 3. Backend (Rust)

- **Module mới `token_refresh`**: Điều phối refresh token qua Claude CLI với cơ chế swap credentials cho saved profiles.
- **Hai command mới**: `refresh_active_token` và `refresh_profile_token` — gọi từ frontend để refresh token bất kỳ tài khoản nào.

### 4. Cải tiến Frontend

- **Hook `useTokenRefresh`**: Hook React mới bọc cả 2 refresh commands kèm loading state.
- **Nâng cấp `ProfileCard`**: Nút "Làm mới Token" inline trên profile hết hạn. Sau thành công, profiles list và usage data tự tải lại.

---

_v1.0.7 loại bỏ việc phải đăng nhập lại cho tài khoản hết hạn. Nếu hiển thị "Hết hạn", chỉ cần một nhấp để khôi phục._
