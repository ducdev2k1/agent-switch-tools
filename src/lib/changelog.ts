/**
 * In-app changelog — bundled so it works fully offline.
 *
 * Newest version first. Each entry carries both locales; the viewer picks the
 * active language at render time. Keep entries concise (user-facing highlights,
 * not commit-level detail) — full release notes live in docs/release-notes-*.
 */

export interface ChangelogEntry {
  version: string
  date: string // ISO yyyy-mm-dd
  en: string[]
  vi: string[]
}

export const CHANGELOG: ChangelogEntry[] = [
  {
    version: '1.0.15',
    date: '2026-07-02',
    en: [
      'Fixed external logins not being detected: logging into a different Claude account via `claude /login` now auto-saves the new account and keeps the previous one intact. This also stops "Save Current" from overwriting the old account with the new one\'s tokens.',
      'The active account\'s backup is now kept fresh on every app open/focus, so the snapshot preserved when you switch accounts is always the latest one.',
      'Account cards show the display name and organization again.',
      'The Auto Session activity log is now a table with dd/mm/yyyy hh:mm timestamps (including reset times, converted to your local timezone).',
      'No more UI flash when re-focusing the window — account and quota info stay on screen and refresh silently, throttled to once per 2 minutes to protect against API rate limits.',
    ],
    vi: [
      'Sửa lỗi không phát hiện đăng nhập ngoài app: login tài khoản Claude khác bằng `claude /login` giờ tự lưu tài khoản mới và giữ nguyên tài khoản cũ. Đồng thời "Lưu Hiện tại" không còn ghi đè tài khoản cũ bằng token của tài khoản mới.',
      'Bản sao lưu của tài khoản đang active giờ được làm mới mỗi lần mở/focus app — snapshot giữ lại khi chuyển tài khoản luôn là bản mới nhất.',
      'Thẻ tài khoản hiển thị lại tên hiển thị và tên tổ chức.',
      'Nhật ký hoạt động của Phiên tự động chuyển sang dạng bảng, ngày giờ định dạng dd/mm/yyyy hh:mm (kể cả giờ reset, quy về múi giờ máy bạn).',
      'Hết nháy giao diện khi focus lại cửa sổ — thông tin tài khoản và quota giữ nguyên trên màn hình, tự làm mới êm phía sau, giới hạn 2 phút/lần để tránh limit API.',
    ],
  },
  {
    version: '1.0.14',
    date: '2026-06-30',
    en: [
      'Fixed token refresh failing with HTTP 404 — the Anthropic OAuth endpoint had moved. This also restores automatic token refresh used by quota updates.',
      'The manual token-refresh button now shows a clear error message when a refresh fails, instead of silently doing nothing.',
      'The token-refresh button is now always visible on expired accounts — no need to hover over the card.',
      'Added this in-app changelog viewer, and new-version release notes now appear in the update dialog.',
    ],
    vi: [
      'Sửa lỗi làm mới token thất bại (HTTP 404) — endpoint OAuth của Anthropic đã đổi. Đồng thời khôi phục cơ chế tự động làm mới token khi cập nhật quota.',
      'Nút làm mới token thủ công giờ hiện thông báo lỗi rõ ràng khi thất bại, thay vì im lặng không phản hồi.',
      'Nút làm mới token luôn hiển thị trên tài khoản đã hết hạn — không cần rê chuột vào thẻ.',
      'Bổ sung trình xem changelog ngay trong app, và release notes của bản mới giờ hiện trong hộp thoại cập nhật.',
    ],
  },
  {
    version: '1.0.13',
    date: '2026-06-29',
    en: [
      'Auto-migrate data across the app rename (claude-tools → agent-switch-tools): profiles, switch history and device identity are restored on first launch.',
      'The old build is now auto-removed when installing the new one, so two versions no longer run side by side.',
      'Temporarily hid Antigravity (Desktop / IDE / CLI) while remaining issues are fixed.',
    ],
    vi: [
      'Tự động chuyển dữ liệu qua lần đổi tên app (claude-tools → agent-switch-tools): profiles, lịch sử chuyển tài khoản và device identity được khôi phục ngay lần mở đầu.',
      'Tự gỡ bản cũ khi cài bản mới, không còn hai bản chạy song song.',
      'Tạm ẩn Antigravity (Desktop / IDE / CLI) trong lúc khắc phục các lỗi còn tồn đọng.',
    ],
  },
  {
    version: '1.0.12',
    date: '2026-06-29',
    en: [
      'Antigravity now supports multi-variant accounts (Desktop, IDE, CLI) under a new quota model.',
      'Cursor and Windsurf are hidden from the dashboard and tray menu.',
      'Session webhook preview shows project, model, branch and message count.',
    ],
    vi: [
      'Antigravity hỗ trợ tài khoản đa biến thể (Desktop, IDE, CLI) theo mô hình quota mới.',
      'Ẩn Cursor và Windsurf khỏi dashboard và tray menu.',
      'Bản xem trước webhook phiên hiển thị project, model, nhánh và số lượng tin nhắn.',
    ],
  },
]
