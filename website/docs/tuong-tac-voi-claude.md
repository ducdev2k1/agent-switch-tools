---
sidebar_position: 3
title: Tương Tác Với Claude
---

# Tương Tác Với Claude — Giải Thích Kỹ Thuật

Tài liệu này chỉ ra **chính xác** những chỗ trong code mà Claude Tools tương tác với hệ thống Claude Code CLI và Anthropic API.

## Tổng quan các điểm tương tác

```
Claude Tools ←→ Hệ thống Claude
─────────────────────────────────

1. ĐỌC file credentials          ~/.claude/.credentials.json
2. ĐỌC file OAuth                ~/.claude.json
3. ĐỌC settings                  ~/.claude/settings.json
4. ĐỌC lịch sử phiên            ~/.claude/history.jsonl
5. ĐỌC session logs              ~/.claude/projects/**/*.jsonl
6. GHI file credentials           ~/.claude/.credentials.json (khi switch)
7. GHI file OAuth                 ~/.claude.json (khi switch)
8. GỌI Anthropic OAuth API        api.anthropic.com/api/oauth/usage
9. CHẠY Claude CLI                claude -p "hi" (khi refresh token)
```

---

## 1. Đọc & Hoán đổi Credentials

**File**: `src-tauri/src/commands/config_commands.rs`

Đây là điểm tương tác **quan trọng nhất** — app trực tiếp ghi đè file `.credentials.json` khi switch profile. Claude Code sẽ tự động dùng credentials mới mà không cần đăng nhập lại.

---

## 2. Gọi Anthropic OAuth API

**File**: `src-tauri/src/commands/quota_commands.rs`

```
GET https://api.anthropic.com/api/oauth/usage
Headers:
  Authorization: Bearer {accessToken}
  anthropic-beta: oauth-2025-04-20
```

**Đây là kết nối INTERNET DUY NHẤT tới Anthropic.** App CHỈ gửi `accessToken` tới domain `api.anthropic.com`.

---

## 3. Background Quota Worker

**File**: `src-tauri/src/quota_refresh_worker.rs`

Worker chạy ngầm mỗi 5 phút, fetch quota cho tất cả profiles, gửi event tới frontend qua `emit("usage-updated")`.

---

## 4. Refresh Token

**File**: `src-tauri/src/commands/token_refresh.rs`

Chạy `claude -p "hi" --max-turns 1` để trigger refresh flow của CLI. Với saved profiles, credentials được swap tạm → refresh → restore lại.

---

## Bản đồ tương tác

```
┌──────────────┐                    ┌──────────────────────┐
│ Claude Tools │                    │ Hệ thống Claude      │
│              │                    │                      │
│  Backend ────┼── ĐỌC ────────►   │ .credentials.json    │
│  (Rust)      │                    │ .claude.json         │
│              │                    │ settings.json        │
│              │                    │ history.jsonl        │
│              │                    │ projects/*.jsonl     │
│              │                    │                      │
│              ├── GHI ────────►   │ .credentials.json    │
│              │   (swap only)      │ .claude.json         │
│              │                    │                      │
│              ├── HTTP GET ───►   │ api.anthropic.com    │
│              │   (quota)          │ /api/oauth/usage     │
│              │                    │                      │
│              ├── EXEC ───────►   │ claude -p "hi"       │
│              │   (refresh)        │ (trigger refresh)    │
└──────────────┘                    └──────────────────────┘
```

### Bảo mật tại mỗi điểm tương tác

| Điểm | Rủi ro | Biện pháp |
|---|---|---|
| Đọc credentials | Token lộ ngoài app | Quyền file 0600, không log token |
| Ghi credentials | Ghi nhầm/mất data | Luôn backup trước khi swap |
| Gọi API | Token bị chặn | Chỉ gọi domain Anthropic chính thức |
| Chạy CLI | Quá trình treo | Timeout, non-blocking async |
| Đọc session logs | Chỉ đọc, không sửa | Read-only, không ghi vào .jsonl |
