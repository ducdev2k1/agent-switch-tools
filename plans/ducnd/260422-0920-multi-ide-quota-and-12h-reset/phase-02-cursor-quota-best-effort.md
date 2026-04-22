# Phase 2: Cursor Quota (Best-Effort)

**Priority:** Medium
**Status:** SKIPPED per user decision (too fragile)
**Scope:** Backend + Frontend types

## Overview

Cursor không có official single-user quota API. Community reverse-engineer endpoint `api2.cursor.sh/auth/full_stripe_profile` và `cursor.com/api/usage`. Làm best-effort — nếu API fail/đổi schema, UI graceful degrade sang "—".

## Key Insights

- Token trong `state.vscdb` key `cursorAuth/accessToken` là JWT
- Community tools dùng trực tiếp JWT làm Bearer (không cần convert session token cho endpoint `full_stripe_profile`)
- Response không có schema official → code phải defensive, parse optional fields
- Cursor có 2 khái niệm: **fast requests** (rate limit 500/month Pro) + **slow requests** (unlimited)
- Endpoint `cursor.com/api/usage?user=<userId>` trả về per-model usage (GPT-4, Claude, etc.) — đây là source đáng tin nhất

## Endpoints to Try (fallback order)

1. `GET https://www.cursor.com/api/usage?user=<sub_from_jwt>` — header `Cookie: WorkosCursorSessionToken=<userId>::<accessToken>`
   - Return: `{"gpt-4": {numRequests, numRequestsTotal, maxRequestUsage, ...}, "gpt-3.5-turbo": {...}, ...}`
2. `GET https://api2.cursor.sh/auth/full_stripe_profile` — header `Authorization: Bearer <accessToken>`
   - Return: subscription tier info

## Requirements

### Functional
- Cursor profile card fetch quota như Anthropic/Antigravity
- Mapping usage sang `UsageLimits` schema (utilization 0-100, resets_at ISO)
- Cache 120s (đồng bộ với cache pattern hiện có)
- Fail gracefully → return `None` → UI show "—"

### Non-functional
- Không làm block khác (timeout 10s)
- Log warn khi API fail nhưng không panic

## Architecture

```
IdeType::Cursor
  └── quota_commands::get_ide_usage()
        └── NEW: cursor::quota::fetch_cursor_quota(token)
              └── Try endpoints in order, return first success
```

**Mapping cursor response → UsageLimits:**
- `fast_requests` (GPT-4 bucket) → `seven_day` (utilization = numRequests/maxRequestUsage * 100)
- Reset time: billing cycle end (từ `full_stripe_profile`) hoặc tháng đầu tiên
- `seven_day_sonnet` nếu có Claude model stats

## Related Code Files

**Create:**
- `src-tauri/src/modules/providers/cursor/quota.rs` — fetch + parse

**Modify:**
- `src-tauri/src/modules/providers/cursor/mod.rs` — add `pub mod quota;`
- `src-tauri/src/commands/quota_commands.rs` — route `IdeType::Cursor` sang cursor quota
- `src-tauri/src/modules/providers/mod.rs` — (nếu cần export)

**No changes:**
- Frontend (hook đã generic, UI đã generic)

## Implementation Steps

1. **Decode JWT để lấy userId (sub claim)**
   - Dùng crate `base64` + `serde_json` (không cần verify signature, chỉ parse payload)
   - JWT format: `header.payload.signature` — decode base64 giữa
   - Extract field `sub` hoặc `user_id`

2. **Create `modules/providers/cursor/quota.rs`:**
   ```rust
   use crate::modules::quota::{UsageBucket, UsageLimits};
   
   pub async fn fetch_cursor_quota(token: &str) -> Option<UsageLimits> {
       let user_id = decode_jwt_sub(token)?;
       let client = &crate::modules::shared::http::CLIENT;
       
       // Try cursor.com/api/usage first
       let url = format!("https://www.cursor.com/api/usage?user={}", user_id);
       let res = client.get(&url)
           .header("Cookie", format!("WorkosCursorSessionToken={}::{}", user_id, token))
           .timeout(std::time::Duration::from_secs(10))
           .send().await.ok()?;
       
       if !res.status().is_success() { return None; }
       let raw: serde_json::Value = res.json().await.ok()?;
       Some(map_to_usage_limits(&raw))
   }
   
   fn decode_jwt_sub(jwt: &str) -> Option<String> { /* base64 decode middle part */ }
   fn map_to_usage_limits(raw: &serde_json::Value) -> UsageLimits { /* defensive parse */ }
   ```

3. **Add cache tương tự claude_cli/quota.rs** — `LazyLock<Mutex<HashMap<u64, ...>>>` TTL 120s.

4. **Wire trong `quota_commands::get_ide_usage`:**
   ```rust
   if ide == IdeType::Cursor {
       return Ok(cursor::quota::fetch_cursor_quota(&token).await);
   }
   ```

5. **Update `Cargo.toml` nếu cần thêm `base64`** (có thể đã có).

6. **Test manual**:
   - Login Cursor trên máy
   - Chạy app, check Cursor card
   - Verify quota hiển thị hoặc "—" nếu API không response

## Todo List

- [ ] Decode JWT helper (no signature verify)
- [ ] `fetch_cursor_quota()` function với fallback endpoints
- [ ] Map response → `UsageLimits`
- [ ] Cache layer (120s TTL)
- [ ] Wire vào `quota_commands::get_ide_usage`
- [ ] Manual test trên Cursor account thật
- [ ] Add warning log khi API fail

## Success Criteria

- Cursor profile có quota hiển thị nếu user login
- Không crash khi API fail → UI show placeholder
- Log message rõ ràng khi endpoint fail (để debug tương lai)

## Risk Assessment

- **High risk of breakage**: Cursor có thể đổi API bất cứ lúc nào
- **Mitigation**: Isolated module, fail gracefully, log warn. Nếu break → chỉ cần update 1 file
- **Response schema unknown**: cần test thực tế, có thể mất 1-2 iteration

## Security Considerations

- JWT không verify signature (chỉ đọc claim) → OK vì chỉ dùng để lấy userId làm query param
- Token truyền qua HTTPS only
- Không log full token (chỉ hash)

## Next Steps

- Research thêm nếu endpoint đổi
- User confirm có muốn bỏ phase này nếu fragility quá cao
