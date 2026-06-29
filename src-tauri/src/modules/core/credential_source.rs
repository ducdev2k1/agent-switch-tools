use std::collections::HashMap;
use std::path::PathBuf;

use crate::modules::core::sqlite_auth::{read_ide_auth_keys, write_ide_auth_keys};

/// Where an IDE/agent's live credentials live.
///
/// Most agents store auth inside a VS Code `state.vscdb` (SQLite). The Antigravity
/// CLI instead keeps a plain JSON token file under `~/.gemini/antigravity-cli/`.
/// This enum lets the profile/switch/quota flow stay agnostic to the backing store.
pub enum CredentialSource {
    /// VS Code `state.vscdb` — keys are rows in the `ItemTable`.
    Vscdb(PathBuf),
    /// Plain JSON file — the whole file content is exposed under the provider's first auth key.
    JsonFile(PathBuf),
}

impl CredentialSource {
    /// True if the underlying store currently exists on disk.
    pub fn exists(&self) -> bool {
        match self {
            CredentialSource::Vscdb(p) | CredentialSource::JsonFile(p) => p.exists(),
        }
    }

    /// Read the requested auth keys into a map.
    ///
    /// - `Vscdb`: one map entry per matching `ItemTable` row.
    /// - `JsonFile`: the entire file content stored under `keys[0]` (the provider's primary key).
    pub fn read(&self, keys: &[&str]) -> Result<HashMap<String, String>, String> {
        match self {
            CredentialSource::Vscdb(db_path) => read_ide_auth_keys(db_path, keys),
            CredentialSource::JsonFile(path) => {
                let primary = keys
                    .first()
                    .ok_or("JsonFile credential source requires a primary key")?;
                let mut map = HashMap::new();
                if path.exists() {
                    let content = std::fs::read_to_string(path)
                        .map_err(|e| format!("read token file: {}", e))?;
                    if !content.trim().is_empty() {
                        map.insert((*primary).to_string(), content);
                    }
                }
                Ok(map)
            }
        }
    }

    /// Persist auth entries into the store.
    ///
    /// - `Vscdb`: UPSERT each entry into `ItemTable`.
    /// - `JsonFile`: write the value of `keys[0]` back to the file (atomic, mode 600).
    pub fn write(&self, primary_key: &str, entries: &HashMap<String, String>) -> Result<(), String> {
        match self {
            CredentialSource::Vscdb(db_path) => write_ide_auth_keys(db_path, entries),
            CredentialSource::JsonFile(path) => {
                let content = entries
                    .get(primary_key)
                    .ok_or("JsonFile write: missing primary key in entries")?;
                write_json_file_atomic(path, content)
            }
        }
    }
}

/// Write `content` to `path` atomically (tmp + rename), with mode 600 on unix.
fn write_json_file_atomic(path: &PathBuf, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create token dir: {}", e))?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content).map_err(|e| format!("write token tmp: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp, path).map_err(|e| format!("rename token file: {}", e))
}
