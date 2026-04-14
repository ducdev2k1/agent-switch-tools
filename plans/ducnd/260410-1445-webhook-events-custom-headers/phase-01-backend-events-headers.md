---
title: "Phase 1: Backend — Events + Custom Headers in Rust"
status: pending
priority: P1
effort: 1h
---

# Phase 1: Backend — Add event_type + custom_headers to Rust webhook command

## Overview
Mở rộng `send_webhook` Rust command để nhận `event_type` (tên event) và `custom_headers` (key-value pairs từ user).

## Related Files
- `src-tauri/src/commands/webhook_commands.rs` — main file cần sửa

## Implementation Steps

### 1. Update `send_webhook` command signature
Thêm 2 params mới:

```rust
#[tauri::command]
pub async fn send_webhook(
    app: tauri::AppHandle,
    url: String,
    secret: Option<String>,
    api_key: Option<String>,
    event_type: Option<String>,           // NEW: "usage_report", "profile_switched", etc.
    custom_headers: Option<Vec<CustomHeader>>,  // NEW: user-defined headers
    test_mode: Option<bool>,
    include_credentials: Option<bool>,
    include_session_usage: Option<bool>,
    member_email: Option<String>,
) -> Result<WebhookResponse, String> {
```

### 2. Define `CustomHeader` struct

```rust
#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomHeader {
    pub key: String,
    pub value: String,
}
```

### 3. Use `event_type` in payload
Thay hardcode `"event": "usage_report"` và `"event": "test"` bằng dynamic value:

- `build_payload()`: nhận `event_type` param, dùng nó thay `"usage_report"`
- Test mode: dùng `event_type` nếu có, fallback `"test"`
- Default event_type nếu None: `"usage_report"`

Update `build_payload` signature:
```rust
async fn build_payload(
    app: &tauri::AppHandle,
    event_type: &str,                    // NEW
    include_credentials: bool,
    include_session_usage: bool,
    member_email: Option<String>,
) -> Result<serde_json::Value, String> {
```

Trong body: `"event": event_type` thay vì hardcode.

### 4. Inject custom headers vào request
Sau khi build request nhưng TRƯỚC HMAC/Bearer headers:

```rust
// Inject user custom headers (before security headers to avoid override)
if let Some(ref headers) = custom_headers {
    for h in headers.iter().take(10) {  // limit 10 headers
        let key = h.key.trim();
        let val = h.value.trim();
        if !key.is_empty() && !is_reserved_header(key) {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                req = req.header(header_name, val);
            }
        }
    }
}
```

### 5. Add reserved header check
Không cho user override security headers:

```rust
fn is_reserved_header(key: &str) -> bool {
    let lower = key.to_lowercase();
    matches!(
        lower.as_str(),
        "content-type" | "x-device-id" | "x-timestamp" | "x-signature" | "authorization" | "host"
    )
}
```

## Todo
- [ ] Define `CustomHeader` struct
- [ ] Add `is_reserved_header` function
- [ ] Update `build_payload` to accept `event_type`
- [ ] Update `send_webhook` params: `event_type`, `custom_headers`
- [ ] Inject custom headers into request
- [ ] Validate: max 10 headers, skip empty keys, skip reserved

## Success Criteria
- `cargo check` passes
- `send_webhook` accepts new params without breaking existing calls (all new params Optional)
- Custom headers injected before security headers
- Reserved headers protected from override
