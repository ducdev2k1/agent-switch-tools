# Sửa lỗi nút cửa sổ (traffic light) không phản hồi trên Linux/Wayland

**Ngày**: 2026-07-09
**Mức độ nghiêm trọng**: Medium (chặn thao tác cơ bản — đóng/thu nhỏ app — nhưng có workaround thủ công)
**Thành phần**: Window lifecycle (main window show/hide/focus), System Tray
**Trạng thái**: Resolved (fix theo workaround đã được cộng đồng Tauri xác nhận, chưa tự verify lại trên máy thật của user)
**Commit**: (chưa commit)

## Vấn đề Gốc

User report: mở app lên, bấm 3 nút điều khiển cửa sổ (thu nhỏ/phóng to/đóng) — lúc bấm được, lúc không. Không có pattern rõ ràng ban đầu.

Chi tiết quan trọng user cung cấp sau đó: **bug xảy ra đúng lúc "đóng hẳn app rồi mở lại"** — app dùng kiểu "đóng xuống tray" (X button không thực sự đóng process, chỉ ẩn window), và sau khi mở lại từ tray, nút đóng không ăn — **cho tới khi bấm fullscreen/maximize thì nút đóng lại hoạt động bình thường**.

Đây chính là chìa khoá xác định root cause: không phải bug ngẫu nhiên, mà là hệ quả trực tiếp của flow `hide() → show()`.

## Sự Thật Tàn Nhẫn

Đây là bug ở tầng dưới code của mình — không sửa được bằng cách đọc kỹ `lib.rs`/`tray.rs` vì logic Rust hoàn toàn đúng (show/unminimize/set_focus gọi đủ, đúng thứ tự). Root cause nằm trong cách GTK (client-side decorations) xử lý input trên Wayland, và đây là bug đã được report lên chính Tauri repo từ lâu, có PR fix ở tầng `tao` nhưng **chưa release** tính đến thời điểm fix (tao 0.34.8 hiện tại, PR fix nhắm tới 0.36.0).

Điểm hay: chi tiết user cung cấp ("bấm fullscreen thì lại đóng được") không phải nhiễu — nó là bằng chứng thực nghiệm trùng khớp 100% với workaround mà một user khác đã report và được xác nhận hoạt động trên GitHub issue thật (`tauri-apps/tauri#11856`). Nếu không có chi tiết đó, có thể đã đi sai hướng: nghi ngờ do khác biệt gọi `show()`/`set_focus()`/`unminimize()` giữa 2 nơi (tray.rs vs single-instance handler) — hướng này đúng một phần (2 nơi gọi không đồng nhất, đã dọn lại cho gọn) nhưng không phải root cause.

## Chi Tiết Kỹ Thuật

### Vấn đề Cụ Thể

- App dùng pattern "đóng xuống tray": `on_window_event` bắt `CloseRequested` → `window.hide()` + `api.prevent_close()`.
- Có 2 nơi gọi lại `show()`/`set_focus()` sau khi ẩn: tray menu "Open" (`tray.rs`) và `tauri_plugin_single_instance` callback (`lib.rs`) — **không đồng nhất**: tray.rs thiếu `unminimize()`.
- Trên Wayland, GTK vẽ title bar bằng client-side decoration (CSD). Sau khi window bị `hide()` rồi `show()` lại, vùng nhận input của các nút CSD không được GTK đăng ký lại đúng — nút vẫn hiển thị nhưng không nhận click, cho tới khi có một sự kiện resize/remap (ví dụ maximize) buộc GTK vẽ lại toolbar.

### Evidence / Investigation

Tra cứu qua GitHub issues + PR của chính Tauri/Tao:
- [`tauri-apps/tauri#13440`](https://github.com/tauri-apps/tauri/issues/13440) — "native window bar's buttons not working (on wayland)": xác nhận chỉ xảy ra trên Wayland (không phải X11), ngay sau khi window mở/hiện lại, double-click title bar là workaround tạm nhưng không ổn định.
- [`tauri-apps/tao#1218`](https://github.com/tauri-apps/tao/pull/1218) — fix thật ở tầng `tao`, merge 29/6/2026, nhắm release **tao 0.36.0**. Project hiện dùng `tao 0.34.8` (qua `tauri 2.10.3`) — **chưa có** trong bất kỳ bản release nào (kiểm tra `cargo search tao` → mới nhất chỉ 0.35.3).
- [`tauri-apps/tauri#11856`](https://github.com/tauri-apps/tauri/issues/11856) — thread quan trọng nhất: user `nekename` mô tả đúng 100% triệu chứng của mình ("window minimises to tray... buttons work the first time, start acting up as soon as it hides itself once... fullscreen makes buttons responsive again"), và user `x11x` confirm workaround: **toggle `set_resizable(false)` → `set_resizable(true)`** ngay khi window nhận `WindowEvent::Focused(true)`.

Ban đầu định implement bằng `set_decorations(false)` → `set_decorations(true)` (đoán rebuild CSD widget sẽ fix), nhưng `code-reviewer` subagent lật lại: cách này **không có ai xác nhận hoạt động** trong thread thật, trong khi `set_resizable` toggle thì có. `set_decorations` cũng nặng hơn (rebuild toàn bộ CSD widget) → rủi ro flicker/compositor snap cao hơn. Đổi qua đúng cách đã verify.

### Quyết Định Thiết Kế

**Không** chờ Tauri/Tao release fix chính thức (chưa có ETA rõ, PR mới merge vào `dev` branch). Áp dụng workaround ở tầng app, scoped `#[cfg(target_os = "linux")]` để không ảnh hưởng macOS/Windows (nơi bug không xảy ra).

Đặt workaround vào `Builder::on_window_event` global handler (xử lý `WindowEvent::Focused(true)`) thay vì gọi thủ công tại từng call-site show() — vì:
1. Tự động cover **mọi** đường show lại cửa sổ (tray "Open", single-instance relaunch, alt-tab, click taskbar...), không cần nhớ gọi helper ở chỗ mới nếu sau này thêm.
2. Khớp chính xác pattern đã được `x11x` confirm hoạt động trên GitHub, giảm rủi ro tự sáng tạo một biến thể chưa ai test.

### Kỹ Thuật Implementation

```rust
.on_window_event(|window, event| match event {
    tauri::WindowEvent::CloseRequested { api, .. } => {
        let _ = window.hide();
        api.prevent_close();
    }
    #[cfg(target_os = "linux")]
    tauri::WindowEvent::Focused(true) => {
        let _ = window.set_resizable(false);
        let _ = window.set_resizable(true);
    }
    _ => {}
})
```

Đồng thời dọn 2 nơi gọi `show()/unminimize()/set_focus()` (tray.rs "Open", lib.rs single-instance) về dùng chung 1 helper `present_main_window()` — tránh lệch hành vi như trước (tray.rs thiếu `unminimize()`).

**Kết Quả:**
- `cargo check` sạch, không warning.
- Không thể chạy test tự động (bug chỉ tái hiện trên GTK/Wayland thật, môi trường dev không có display server) — verify hành vi thực tế phải chờ user xác nhận trên máy Linux/Wayland của họ.

## Bài Học Rút Ra

1. **Chi tiết "tái hiện được bằng cách nào" từ user quan trọng hơn log/code review khi bug liên quan tầng OS/windowing.** Câu "bấm fullscreen xong lại đóng được" chính là dữ liệu quyết định, đến sau khi đã tưởng root cause nằm ở thứ tự gọi API trong Rust.
2. **Đừng tự tin vào một fix "nghe hợp lý" (`set_decorations` toggle) khi có sẵn một fix đã được người dùng thật xác nhận (`set_resizable` toggle) — tra kỹ comment thật trên issue thay vì suy luận từ tên API.** `code-reviewer` subagent bắt được điểm này bằng cách đọc thẳng comment trên GitHub thay vì chỉ đọc mô tả issue.
3. **Gắn fix vào global event handler thay vì từng call-site cụ thể** khi bug có thể trigger từ nhiều đường (tray, single-instance, OS focus) — giảm rủi ro bỏ sót đường mới trong tương lai.

## Rủi Ro Còn Lại

- Chưa tự verify trên máy Linux/Wayland thật (môi trường dev hiện tại không có GUI để bấm thử) — cần user xác nhận sau khi build lại.
- Đây là workaround, không phải fix gốc — khi `tao` release bản có PR #1218 (dự kiến 0.36.0), nên gỡ bỏ workaround và bump dependency, tránh double-toggle không cần thiết về sau.
- Không rõ workaround có tác dụng phụ trên các Wayland compositor khác nhau (GNOME/KDE/Sway...) hay chỉ đã test trên compositor của người report — vẫn tốt hơn tình trạng cũ vì không tệ hơn (không resize gì cả nếu bug không xảy ra).

## Các Bước Tiếp Theo

1. User build lại app trên máy Linux/Wayland, xác nhận nút đóng/thu nhỏ/phóng to luôn bấm được sau khi ẩn/mở lại từ tray.
2. Theo dõi release `tao` — khi có bản chứa PR #1218, gỡ workaround `set_resizable` toggle, bump `tauri`/`tao` version.
3. Nếu vẫn còn tái hiện trên một số Wayland compositor cụ thể, thu thập tên compositor (GNOME/KDE/Hyprland...) để report ngược lên `tauri-apps/tauri#13440`.

---

## File Được Thay Đổi (Liên Quan)

- `src-tauri/src/lib.rs` — thêm `present_main_window()` helper, thêm nhánh `Focused(true)` trong `on_window_event` (Linux-only)
- `src-tauri/src/tray.rs` — "Open" menu item dùng `present_main_window()` thay vì gọi `show()/set_focus()` riêng
