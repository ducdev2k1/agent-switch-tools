// Abstraction over where the ACTIVE Claude Code credential blob lives.
//
// Linux/Windows: the file `<config_dir>/.credentials.json` (unchanged from before).
// macOS: the login Keychain. Claude Code 2.x keys each config dir's entry by a path hash —
//   service = "Claude Code-credentials-<sha256(config_dir)[:8 hex]>"
// Older CLIs used the un-suffixed global name "Claude Code-credentials". The stored value is the
// identical JSON blob (`{"claudeAiOauth":{...}}`) that the file would hold.
//
// Backend selection uses the RUNTIME gate `cfg!(target_os = "macos")`, not `#[cfg]`, so the whole
// keychain branch is type-checked by `cargo check` on any host and simply never executes off-macOS.

use std::path::PathBuf;
use std::process::Command;

use sha2::{Digest, Sha256};

const GLOBAL_SERVICE: &str = "Claude Code-credentials";
const KEYCHAIN_WRITE_RETRIES: u32 = 3;

/// Storage for the active Claude credential at a given config dir (`~/.claude`).
pub struct ActiveStore {
    config_dir: PathBuf,
    creds_file: PathBuf,
}

impl ActiveStore {
    pub fn new(config_dir: PathBuf) -> Self {
        let creds_file = config_dir.join(".credentials.json");
        Self { config_dir, creds_file }
    }

    /// Production constructor: config dir = `~/.claude`.
    pub fn resolve(app: &tauri::AppHandle) -> Result<Self, String> {
        Ok(Self::new(crate::modules::shared::paths::claude_dir(app)?))
    }

    /// Path to the credentials file (used where a file path is still required, e.g. profile IO).
    pub fn creds_file(&self) -> &PathBuf {
        &self.creds_file
    }

    fn use_keychain() -> bool {
        cfg!(target_os = "macos")
    }

    /// "Claude Code-credentials-<sha256(config_dir)[:8]>".
    fn hashed_service(&self) -> String {
        format!("{GLOBAL_SERVICE}-{}", keychain_suffix(&self.config_dir))
    }

    /// Read the active credential JSON blob, or None if there is none.
    /// macOS: hashed keychain slot → global keychain slot → file. Otherwise: file.
    pub fn read_active(&self) -> Option<String> {
        if Self::use_keychain() {
            if let Some(blob) = read_keychain_blob(&self.hashed_service()) {
                return Some(blob);
            }
            if let Some(blob) = read_keychain_blob(GLOBAL_SERVICE) {
                return Some(blob);
            }
        }
        read_file_blob(&self.creds_file)
    }

    pub fn active_exists(&self) -> bool {
        self.read_active().is_some()
    }

    /// Write the active credential JSON blob.
    /// macOS: write to the keychain slot that currently holds the entry (hashed → global →
    /// default hashed), confirm the write landed, and mirror to the file only if the file already
    /// exists (keeps a copy readable while the login keychain is locked). Otherwise: file only.
    pub fn write_active(&self, blob: &str) -> Result<(), String> {
        if Self::use_keychain() {
            // Store the exact JSON with no surrounding whitespace: a trailing newline (typical in
            // a credentials.json copied off another machine) makes `find-generic-password -w`
            // print the value hex-encoded, which breaks the write verification and any reader.
            let blob = blob.trim();
            let service = if keychain_service_exists(&self.hashed_service()) {
                self.hashed_service()
            } else if keychain_service_exists(GLOBAL_SERVICE) {
                GLOBAL_SERVICE.to_string()
            } else {
                self.hashed_service()
            };
            let account = read_keychain_account(&service).unwrap_or_else(current_user);
            let mut wrote = write_keychain_blob(&service, &account, blob);

            // An in-place `-U` update keeps the existing item's ACL. When that item was created by
            // another app (e.g. the Claude Code CLI on first login, or restored from a profile
            // shared off another machine), its ACL can block a silent rewrite and macOS pops the
            // keychain-password dialog — leaving the write unverified. Recover by deleting the slot
            // and recreating it with an app-owned `-A` ACL so future writes never prompt.
            // Safe: the blob is in hand and the profile file still holds a copy.
            if wrote.is_err() {
                let _ = delete_keychain(&service);
                wrote = write_keychain_blob(&service, &account, blob);
            }

            // Mirror to the file only where one already exists — never create a plaintext copy.
            if self.creds_file.exists() {
                let _ = write_file_blob(&self.creds_file, blob);
            }
            if let Err(detail) = &wrote {
                // Keychain write failed: fall back to the file if we have one.
                if self.creds_file.exists() {
                    return write_file_blob(&self.creds_file, blob);
                }
                return Err(format!(
                    "Failed to write credential to macOS Keychain: {detail}"
                ));
            }
            return Ok(());
        }
        write_file_blob(&self.creds_file, blob)
    }

    /// Remove the active credential from every backend.
    pub fn delete_active(&self) -> Result<(), String> {
        if Self::use_keychain() {
            let _ = delete_keychain(&self.hashed_service());
            let _ = delete_keychain(GLOBAL_SERVICE);
        }
        if self.creds_file.exists() {
            std::fs::remove_file(&self.creds_file).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

// ========== Keychain helpers (macOS `security` CLI; compile everywhere, run only via cfg!) ==========

const SECURITY_TIMEOUT_SECS: u64 = 15;

/// Run `security` with a hard timeout. macOS can park a keychain call behind a SecurityAgent
/// dialog (locked keychain, ACL confirmation) — in an app or CI context nobody may ever dismiss
/// it, so a bounded wait turns an indefinite hang into a diagnosable error.
fn run_security(args: &[&str]) -> Result<std::process::Output, String> {
    let mut child = Command::new("security")
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not run `security`: {e}"))?;
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(SECURITY_TIMEOUT_SECS);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|e| e.to_string()),
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "`security {}` timed out after {SECURITY_TIMEOUT_SECS}s — likely blocked on a keychain dialog",
                        args.first().unwrap_or(&"")
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// Claude's keychain suffix for a config dir = `sha256(path)[:8]` (hex, first 4 bytes).
fn keychain_suffix(config_dir: &PathBuf) -> String {
    let mut hasher = Sha256::new();
    hasher.update(config_dir.to_string_lossy().as_bytes());
    hasher
        .finalize()
        .iter()
        .take(4)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn read_keychain_blob(service: &str) -> Option<String> {
    run_security(&["find-generic-password", "-s", service, "-w"])
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|blob| !blob.is_empty())
}

fn keychain_service_exists(service: &str) -> bool {
    run_security(&["find-generic-password", "-s", service, "-w"])
        .map(|out| out.status.success() && !out.stdout.is_empty())
        .unwrap_or(false)
}

/// Read the account name of a keychain entry so a write targets the same (service, account).
/// The attribute dump (no `-w`) prints a line like `"acct"<...>="<name>"`.
fn read_keychain_account(service: &str) -> Option<String> {
    let out = run_security(&["find-generic-password", "-s", service])
        .ok()
        .filter(|out| out.status.success())?;
    let dump = String::from_utf8_lossy(&out.stdout);
    let line = dump.lines().find(|line| line.contains("\"acct\""))?;
    // Split on the FIRST `=` only — the account value may itself contain `=`.
    let value = line.split_once('=')?.1.trim().trim_matches('"');
    (!value.is_empty() && value != "<NULL>").then(|| value.to_string())
}

/// Write a blob to a keychain entry via `add-generic-password -U -A`, then read it back by service
/// to confirm the write landed on the slot Claude reads. Retries to ride out a transient lock.
///   -U  upsert the existing (service, account) item in place (no duplicate).
///   -A  allow any app of this user to read without a prompt — required for an unsigned local
///       build so the keychain dialog never fires; token is the same one Claude Code already keeps.
/// The blob is passed as the `-w` argument (not stdin): `add-generic-password` has no stdin path
/// for the value, so it is briefly visible in this user's process list while `security` runs —
/// acceptable on a personal machine and the same approach a production macOS switcher ships.
///
/// Returns `Ok(())` once a write is confirmed, or `Err(detail)` with the last failure reason
/// (the `security` stderr, or why the read-back mismatched) so callers can surface a real cause.
fn write_keychain_blob(service: &str, account: &str, blob: &str) -> Result<(), String> {
    if account.trim().is_empty() {
        return Err("could not determine the login account name (USER/LOGNAME unset and `id -un` failed)".to_string());
    }
    let mut args = vec!["add-generic-password", "-U"];
    // `-A` works silently on real Macs, but CI's SecurityAgent cannot confirm the open-access
    // grant headlessly and parks the call behind a dialog that never shows. Tests set this var
    // so the rest of the write path still runs against a real keychain; it is never set for users.
    if std::env::var_os("AGENT_SWITCH_TEST_NO_KEYCHAIN_ACL").is_none() {
        args.push("-A");
    }
    args.extend(["-s", service, "-a", account, "-w", blob]);

    let mut last_error = String::new();
    for attempt in 0..KEYCHAIN_WRITE_RETRIES {
        match run_security(&args) {
            Ok(out) if out.status.success() => {
                // `read_keychain_blob` already trims, and `security -w` appends its own newline,
                // so compare against the trimmed blob — a trailing '\n' in the source file must
                // not make a landed write look like a mismatch.
                if read_keychain_blob(service).as_deref() == Some(blob.trim()) {
                    return Ok(());
                }
                last_error = "write reported success but read-back did not match (keychain may be locked or ACL denied read access)".to_string();
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                last_error = if stderr.is_empty() {
                    format!("`security` exited with {}", out.status)
                } else {
                    stderr
                };
            }
            Err(e) => last_error = e,
        }
        if attempt + 1 < KEYCHAIN_WRITE_RETRIES {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
    Err(last_error)
}

fn delete_keychain(service: &str) -> bool {
    run_security(&["delete-generic-password", "-s", service])
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Current login name: `$USER` → `$LOGNAME` → `id -un`.
fn current_user() -> String {
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() {
            return user;
        }
    }
    if let Ok(user) = std::env::var("LOGNAME") {
        if !user.is_empty() {
            return user;
        }
    }
    Command::new("id")
        .arg("-un")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_default()
}

// ========== File backend ==========

fn read_file_blob(path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .filter(|text| !text.trim().is_empty())
}

/// Atomically write a credential blob (temp write + rename), owner-only `0600`. Preserves existing
/// perms on overwrite; a fresh file keeps `0600` so the OAuth token is never world-readable.
fn write_file_blob(path: &PathBuf, blob: &str) -> Result<(), String> {
    let tmp = path.with_extension("json.tmp");
    let existing_perms = std::fs::metadata(path).ok().map(|m| m.permissions());

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        // 0600 applied at open, before any bytes land.
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        f.write_all(blob.as_bytes()).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&tmp, blob).map_err(|e| e.to_string())?;
    }

    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })?;

    #[cfg(unix)]
    if let Some(perms) = existing_perms {
        let _ = std::fs::set_permissions(path, perms);
    }
    let _ = existing_perms;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On non-macOS, ActiveStore is a pure file backend: round-trip read/write/exists/delete.
    /// (On macOS these paths route through the keychain — covered by `keychain_tests` below.)
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn file_backend_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ActiveStore::new(tmp.path().to_path_buf());

        assert!(!store.active_exists());
        assert_eq!(store.read_active(), None);

        let blob = r#"{"claudeAiOauth":{"accessToken":"tok"}}"#;
        store.write_active(blob).unwrap();

        assert!(store.active_exists());
        assert_eq!(store.read_active().as_deref(), Some(blob));

        store.delete_active().unwrap();
        assert!(!store.active_exists());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn empty_or_whitespace_file_reads_as_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ActiveStore::new(tmp.path().to_path_buf());
        std::fs::write(store.creds_file(), "   \n").unwrap();
        assert_eq!(store.read_active(), None);
        assert!(!store.active_exists());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn write_sets_owner_only_perms_on_fresh_file() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ActiveStore::new(tmp.path().to_path_buf());
        store.write_active("{}").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(store.creds_file()).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    /// Keychain service name must match Claude Code 2.x: sha256(path)[:8] hex, stable per dir.
    #[test]
    fn keychain_suffix_is_8_hex_chars_and_deterministic() {
        let dir = PathBuf::from("/Users/someone/.claude");
        let a = keychain_suffix(&dir);
        let b = keychain_suffix(&dir);
        assert_eq!(a, b);
        assert_eq!(a.len(), 8);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Real-keychain integration tests. They mutate the DEFAULT keychain (and `delete_active`
    /// also clears the un-suffixed global "Claude Code-credentials" slot a real Claude login
    /// uses), so they only run when KEYCHAIN_TESTS=1 — set in CI where a throwaway keychain is
    /// created and made default. Never set it on a personal Mac.
    #[cfg(target_os = "macos")]
    mod keychain_tests {
        use super::super::{run_security, ActiveStore};

        fn enabled() -> bool {
            std::env::var("KEYCHAIN_TESTS").map(|v| v == "1").unwrap_or(false)
        }

        /// The exact scenario a shared profile hits: no `.credentials.json` on disk, blob read
        /// from a file with a trailing newline, keychain slot is the only write target.
        #[test]
        fn keychain_write_read_delete_round_trip() {
            if !enabled() {
                eprintln!("skipped: set KEYCHAIN_TESTS=1 (CI-only, mutates the default keychain)");
                return;
            }
            let tmp = tempfile::tempdir().unwrap();
            let store = ActiveStore::new(tmp.path().to_path_buf());
            assert!(!store.creds_file().exists());

            // Trailing newline like a credentials.json copied off another machine.
            let blob = "{\"claudeAiOauth\":{\"accessToken\":\"tok-shared\"}}\n";
            store.write_active(blob).unwrap();

            assert!(store.active_exists());
            assert_eq!(store.read_active().as_deref(), Some(blob.trim()));
            // No plaintext copy may appear as a side effect of a keychain write.
            assert!(!store.creds_file().exists());

            // Second write exercises the in-place `-U` update on an existing item.
            let blob2 = r#"{"claudeAiOauth":{"accessToken":"tok-2"}}"#;
            store.write_active(blob2).unwrap();
            assert_eq!(store.read_active().as_deref(), Some(blob2));

            store.delete_active().unwrap();
            assert!(!store.active_exists());
        }

        /// A stale/foreign slot that `-U` can't cleanly update must be recovered by the
        /// delete-and-recreate fallback rather than surfacing an error.
        #[test]
        fn keychain_recreates_slot_and_succeeds_after_existing_entry() {
            if !enabled() {
                eprintln!("skipped: set KEYCHAIN_TESTS=1 (CI-only, mutates the default keychain)");
                return;
            }
            let tmp = tempfile::tempdir().unwrap();
            let store = ActiveStore::new(tmp.path().to_path_buf());

            // Seed the slot out-of-band under a DIFFERENT account name, the way another
            // machine's app or CLI would have — the write must still land and be readable.
            let service = store.hashed_service();
            let seeded = run_security(&[
                "add-generic-password", "-U", "-s", &service, "-a", "someone-else", "-w", "old",
            ])
            .map(|out| out.status.success())
            .unwrap_or(false);
            assert!(seeded, "failed to seed keychain entry");

            let blob = r#"{"claudeAiOauth":{"accessToken":"tok-new"}}"#;
            store.write_active(blob).unwrap();
            assert_eq!(store.read_active().as_deref(), Some(blob));

            store.delete_active().unwrap();
            assert!(!store.active_exists());
        }
    }
}
