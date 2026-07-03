# Lưu trữ Active Credential trên macOS Keychain

**Ngày**: 2026-07-02 17:32  
**Mức độ nghiêm trọng**: Medium  
**Thành phần**: Active Credential Store, Account Switching  
**Trạng thái**: Resolved (chưa verify trên Mac thực)  
**Commit**: f821fe7

## Vấn đề Gốc

Cơ chế switch account hiện tại chỉ đọc/ghi active credential từ file `~/.claude/.credentials.json` cho mọi HĐH. Trên macOS **hoàn toàn sai** vì:

- Claude Code CLI 2.x (macOS) lưu credential trong login Keychain, không trong file
- Service name: `Claude Code-credentials-<sha256(config_dir)[:8]>`
- Hệ quả: list endpoint show "no active account", save/switch/reconcile/refresh đều trượt đối tượng — không thể quản lý account trên Mac

**Phát hiện qua**: Tham khảo repo github.com/hoangpm96/ai-switcher (macOS switcher đã production) — đó là thực tế của Keychain trên Mac, không phải giả định.

## Sự Thật Tàn Nhẫn

Đây là một tình huống khá phiền phức: logic account switching hiện tại chỉ hoạt động trên Linux/Windows, hoàn toàn vô dụng trên macOS. Bất kỳ người dùng Mac nào dùng feature này sẽ gặp hiện tượng "account ảo" — không nhìn thấy cái mình vừa save, không thể switch, không thể refresh token.

Tệ hơn là phát hiện này xảy ra sau khi code already landed (v1.0.15). May mắn là chưa ai report issue từ Mac users, nhưng nó là một time bomb.

Điểm khó chịu nhất: em phải làm quyết định thiết kế trên máy Linux (không có Mac để test), nhưng phải đảm bảo code chạy đúng trên Mac. Rủi ro cao.

## Chi Tiết Kỹ Thuật

### Vấn đề Cụ Thể
- Mỗi OS quản lý credential khác nhau:
  - **macOS**: Keychain service `Claude Code-credentials-{hash}`, không có file `.credentials.json`
  - **Linux/Windows**: File `~/.claude/.credentials.json`, không Keychain
- Current code: hardcoded file path → mất credential trên Mac
- Trigger: list/save/switch/reconcile/refresh/quota/tray/priming/webhook đều cần active credential

### Kỹ thuật Keychain
Học từ ai-switcher:
- Service name có hash: `sha256(config_dir)[:8]` để tránh conflict
- Write: `security add-generic-password -U -A -w <json> -s <service> -a <account>`
  - `-U`: update nếu tồn tại
  - `-A`: no ACL prompt
  - Read-back confirm + retry (Keychain có khi chậm)
- Fallback: nếu keychain fail, mirror đến file `~/.claude.json` cho DarkWake/offline

### Quyết Định Thiết Kế

**Hai hướng tiềm năng:**
- **Hướng A** (ai-switcher): Isolated `CLAUDE_CONFIG_DIR` per account → mỗi account riêng biệt folder
  - Ưu: hoàn toàn sạch, không conflict
  - Nhược: đại phẫu kiến trúc, refactor lớn, máy Linux không thể test
- **Hướng B** (chọn): Thêm Keychain backend cho active slot, giữ kiến trúc single `~/.claude`
  - Ưu: isolated nhánh macOS via `cfg!()`, Linux/Windows không đổi
  - Nhược: thêm abstraction, phải quản lý Keychain↔file mirror

**Chọn Hướng B vì:**
1. Máy dev là Linux, không có Mac để test/iterate
2. Hướng A yêu cầu breaking changes, quá rủi ro khi không thể verify thực tế
3. Hướng B là bước an toàn: cô lập feature macOS, không impact Linux/Windows

### Kỹ Thuật Implementation

**Abstraction `ActiveStore`:**
```rust
// Signature tựa như:
pub trait ActiveStore: Send + Sync {
    async fn load(&self) -> Result<Option<OAuthAccount>>;
    async fn save(&self, account: &OAuthAccount) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}
```

**Runtime OS Check (MẤU CHỐT):**
```rust
#[cfg(target_os = "macos")]
let store = Box::new(KeychainActiveStore::new(...));

#[cfg(not(target_os = "macos"))]
let store = Box::new(FileActiveStore::new(...));
```

**Vì sao dùng `cfg!()` runtime thay `#[cfg]` compile-time?**
- `cfg!()` là runtime check → entire code path vẫn compile-check trên Linux
- `#[cfg]` là compile-time → dead code trên Linux không thể verify type-check
- **Điểm mấu chốt**: từ máy Linux, em có thể `cargo check` on target `x86_64-apple-darwin` và đảm bảo nó type-check đúng (không chạy, nhưng compile được)

**Refactor routes:**
1. List endpoint: fetch từ ActiveStore
2. Save/Switch: write qua ActiveStore
3. Token Refresh: load từ ActiveStore, parse, update, save lại
4. Reconcile: compare list vs ActiveStore → sync
5. Quota Worker: load account từ ActiveStore
6. Tray: fetch active từ ActiveStore
7. Priming: pass ActiveStore vào
8. Webhook: fetch active từ payload or ActiveStore
9. oauth.rs: blob-in/blob-out (không hardcoded file path)
10. config.rs: blob-in/blob-out (refresh não phải đọc file)

**Code Review phát hiện:**
- Missed touchpoint: `webhook_commands.rs` lúc đầu không route qua ActiveStore → fixed

**Kết Quả:**
- `cargo check` sạch
- `cargo test`: 26 tests pass trên Linux
- No changes to saved profile files (`.profiles/`, `.config.json`)

## Bài Học Rút Ra

1. **Tham khảo implementation production của người khác tiết kiệm rất nhiều thời gian.**
   - ai-switcher đã solve bài toán này, việc bỏ 2 giờ tìm hiểu nó tiết kiệm 20 giờ trial-and-error
   - Ví dụ: nếu không biết service name có hash, sẽ mất vài ngày debug trên Mac

2. **Dùng `cfg!()` runtime thay `#[cfg]` compile-time để giữ code cross-platform compile-check.**
   - Điều này cho phép từ Linux, em vẫn `cargo check --target x86_64-apple-darwin` và verify macOS code
   - Nếu dùng `#[cfg]`, nhánh macOS sẽ hoàn toàn bị ignore trên Linux → không biết compile error gì đến khi ship

3. **Abstraction là bạn khi phải support divergent behavior.**
   - `ActiveStore` trait che giấu Keychain/File differences → 10 callsites không biết platform-specific logic
   - Dễ test (mock), dễ verify type-check

4. **Verify thủ công là cần thiết khi không có target machine.**
   - Code này chưa được chạy trên Mac thật
   - Trước khi ship bản macOS, phải verify 3 điểm:
     1. Service name hash tính đúng slot active
     2. `~/.claude.json` vẫn chứa `oauthAccount.emailAddress` sau save
     3. Keychain ACL không prompt (flag `-A` có hoạt động)

## Rủi Ro Còn Lại

**Critical**: Nhánh macOS Keychain chỉ **compile-check**, chưa **runtime-test** trên Mac thực.
- Code type-checks ✓
- Tests pass on Linux ✓
- Keychain logic chưa verify on macOS ✗

**Mitigation**: Phải có **test/QA thủ công trên Mac** trước khi official release (chẳng hạn v1.0.17).

## Các Bước Tiếp Theo

1. **Tìm Mac dev/ QA để verify 3 điểm trên** → nếu fail, debug và ship hotfix
2. **Nếu verify pass**: document lại exact service name + Keychain ACL behavior trong code comment
3. **Future**: cân nhắc viết integration test mô phỏng Keychain (mock `security` command)
4. **Feedback từ users**: nếu có Mac user report issue, phải check Keychain state (`security find-generic-password`)

---

## File Được Thay Đổi (Liên Quan)
- `src-tauri/src/modules/shared/active_store.rs` (new, 328 lines)
- `src-tauri/src/commands/config_commands.rs` (refactor, -40 lines)
- `src-tauri/src/commands/quota_commands.rs` (refactor, -41 lines)
- `src-tauri/src/commands/token_refresh.rs` (refactor, -27 lines)
- `src-tauri/src/commands/webhook_commands.rs` (refactor, -39 lines)
- `src-tauri/src/modules/providers/claude_cli/{config,oauth,reconcile}.rs` (blob-in/out)
- `src-tauri/src/{priming,quota_refresh_worker,tray}.rs` (route)
