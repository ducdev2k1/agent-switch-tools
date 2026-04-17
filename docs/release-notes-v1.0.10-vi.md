# Agent Switch Tools v1.0.10 - Ghi chú phát hành

**Agent Switch Tools v1.0.10** là bản cập nhật đổi tên lớn và tái cấu trúc toàn diện. Ứng dụng — trước đây tên là **Claude Tools** — giờ đây hỗ trợ switch tài khoản cho nhiều AI coding agent: **Claude Code**, **Cursor**, **Windsurf** và **Antigravity**.

## Có gì mới?

### 1. Đổi tên: Claude Tools → Agent Switch Tools

- **Tên mới**: Vì app không chỉ dành riêng cho Claude Code nữa, chúng tôi đổi tên để phản ánh đúng phạm vi — quản lý nhiều tài khoản AI coding agent từ một nơi.
- **Identifier mới**: Bundle identifier đổi thành `com.ducdev2k1.agent-switch-tools`.
- **Repo đã chuyển**: GitHub mới tại `github.com/ducdev2k1/agent-switch-tools`.
- **UI cập nhật**: Tiêu đề cửa sổ, tooltip tray, trang settings và mọi nơi đều hiển thị "Agent Switch Tools".

### 2. Switch tài khoản Đa IDE

- **Hỗ trợ Cursor**: Switch giữa nhiều tài khoản Cursor bằng cách backup và restore auth keys từ `state.vscdb` (SQLite) của Cursor.
- **Hỗ trợ Windsurf**: Quản lý đầy đủ tài khoản Windsurf IDE với trích xuất email từ protobuf.
- **Hỗ trợ Antigravity**: Quản lý nhiều tài khoản Antigravity với trích xuất email từ JSON field.
- **Tự động phát hiện**: App tự động phát hiện các IDE đã cài trên máy và hiển thị trong dashboard.
- **Profile riêng cho từng IDE**: Mỗi IDE có kho profile độc lập — không lẫn lộn dữ liệu.

### 3. Cấu trúc lưu trữ thống nhất

- **Thư mục gốc mới**: Toàn bộ dữ liệu app giờ nằm trong `~/.agent-switch-tools/` (trước đây là `~/.claude/.claude-tools/`).
- **Bố cục đồng nhất**: Mỗi agent/IDE có subfolder riêng với cấu trúc giống nhau:
  ```
  ~/.agent-switch-tools/
  ├── device.json           ← định danh thiết bị (toàn cục)
  ├── claude/               ← dữ liệu Claude Code
  │   ├── meta.json
  │   └── profiles/{email}/
  ├── cursor/               ← dữ liệu Cursor IDE
  │   └── profiles/{email}/
  ├── windsurf/             ← dữ liệu Windsurf IDE
  │   └── profiles/{email}/
  └── antigravity/          ← dữ liệu Antigravity IDE
      └── profiles/{email}/
  ```
- **Tự động migrate**: Khi lần đầu chạy v1.0.10, app tự động chuyển dữ liệu từ mọi vị trí cũ (`~/.claude/.claude-tools/`, `~/.claude-tools/`, hoặc file phẳng `~/.claude/`). Không cần thao tác thủ công.

### 4. Tray Menu cải tiến

- **Section đa agent**: Tray menu giờ hiển thị từng section riêng cho Claude Code và mỗi IDE đã cài — mỗi section có tài khoản đang active và danh sách switch nhanh.
- **Phát hiện trực tiếp**: Chỉ IDE đã cài mới xuất hiện trong tray; IDE chưa cài tự động bị ẩn.

---

_v1.0.10 biến app từ công cụ chuyên dụng cho Claude Code thành trình quản lý tài khoản AI coding agent đa năng. Một app, mọi agent._
