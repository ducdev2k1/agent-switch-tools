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
            let service = if keychain_service_exists(&self.hashed_service()) {
                self.hashed_service()
            } else if keychain_service_exists(GLOBAL_SERVICE) {
                GLOBAL_SERVICE.to_string()
            } else {
                self.hashed_service()
            };
            let account = read_keychain_account(&service).unwrap_or_else(current_user);
            let wrote = write_keychain_blob(&service, &account, blob);

            // Mirror to the file only where one already exists — never create a plaintext copy.
            if self.creds_file.exists() {
                let _ = write_file_blob(&self.creds_file, blob);
            }
            if wrote {
                return Ok(());
            }
            // Keychain write failed: fall back to the file if we have one.
            if self.creds_file.exists() {
                return write_file_blob(&self.creds_file, blob);
            }
            return Err("Failed to write credential to macOS Keychain".to_string());
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
    Command::new("security")
        .args(["find-generic-password", "-s", service, "-w"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|blob| !blob.is_empty())
}

fn keychain_service_exists(service: &str) -> bool {
    Command::new("security")
        .args(["find-generic-password", "-s", service, "-w"])
        .output()
        .map(|out| out.status.success() && !out.stdout.is_empty())
        .unwrap_or(false)
}

/// Read the account name of a keychain entry so a write targets the same (service, account).
/// The attribute dump (no `-w`) prints a line like `"acct"<...>="<name>"`.
fn read_keychain_account(service: &str) -> Option<String> {
    let out = Command::new("security")
        .args(["find-generic-password", "-s", service])
        .output()
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
fn write_keychain_blob(service: &str, account: &str, blob: &str) -> bool {
    for attempt in 0..KEYCHAIN_WRITE_RETRIES {
        let wrote = Command::new("security")
            .args([
                "add-generic-password",
                "-U",
                "-A",
                "-s",
                service,
                "-a",
                account,
                "-w",
                blob,
            ])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false);
        if wrote && read_keychain_blob(service).as_deref() == Some(blob) {
            return true;
        }
        if attempt + 1 < KEYCHAIN_WRITE_RETRIES {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
    false
}

fn delete_keychain(service: &str) -> bool {
    Command::new("security")
        .args(["delete-generic-password", "-s", service])
        .output()
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

    #[test]
    fn empty_or_whitespace_file_reads_as_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ActiveStore::new(tmp.path().to_path_buf());
        std::fs::write(store.creds_file(), "   \n").unwrap();
        assert_eq!(store.read_active(), None);
        assert!(!store.active_exists());
    }

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
}
