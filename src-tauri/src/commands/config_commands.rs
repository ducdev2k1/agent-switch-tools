use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// Resolve đường dẫn thư mục ~/.claude/ trên mọi nền tảng
fn claude_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .home_dir()
        .map(|h| h.join(".claude"))
        .map_err(|e| format!("Cannot resolve home directory: {}", e))
}

// ========== Credential Profile Types ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialInfo {
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
    pub expires_at: Option<i64>,
    pub scopes: Vec<String>,
    pub organization_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialProfile {
    pub name: String,
    pub is_active: bool,
    pub info: CredentialInfo,
}

/// Đọc metadata từ file credentials JSON
fn read_credential_info(path: &PathBuf) -> CredentialInfo {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return CredentialInfo {
                subscription_type: None,
                rate_limit_tier: None,
                expires_at: None,
                scopes: vec![],
                organization_uuid: None,
            }
        }
    };

    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            return CredentialInfo {
                subscription_type: None,
                rate_limit_tier: None,
                expires_at: None,
                scopes: vec![],
                organization_uuid: None,
            }
        }
    };

    let oauth = v.get("claudeAiOauth");

    CredentialInfo {
        subscription_type: oauth
            .and_then(|o| o.get("subscriptionType"))
            .and_then(|s| s.as_str())
            .map(String::from),
        rate_limit_tier: oauth
            .and_then(|o| o.get("rateLimitTier"))
            .and_then(|s| s.as_str())
            .map(String::from),
        expires_at: oauth
            .and_then(|o| o.get("expiresAt"))
            .and_then(|s| s.as_i64()),
        scopes: oauth
            .and_then(|o| o.get("scopes"))
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        organization_uuid: v
            .get("organizationUuid")
            .and_then(|s| s.as_str())
            .map(String::from),
    }
}

// ========== Commands ==========

/// Liệt kê tất cả credential profiles (active + saved)
#[tauri::command]
pub async fn list_credential_profiles(
    app: tauri::AppHandle,
) -> Result<Vec<CredentialProfile>, String> {
    let dir = claude_dir(&app)?;
    let mut profiles: Vec<CredentialProfile> = vec![];

    // Đọc active profile (.credentials.json)
    let active_path = dir.join(".credentials.json");
    if active_path.exists() {
        let info = read_credential_info(&active_path);
        profiles.push(CredentialProfile {
            name: "Active".to_string(),
            is_active: true,
            info,
        });
    }

    // Scan saved profiles (.credentials-[name].json)
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with(".credentials-") && filename.ends_with(".json") {
                // Trích xuất tên từ filename: .credentials-Work.json → Work
                let name = filename
                    .strip_prefix(".credentials-")
                    .and_then(|s| s.strip_suffix(".json"))
                    .unwrap_or("")
                    .to_string();

                if !name.is_empty() {
                    let info = read_credential_info(&entry.path());
                    profiles.push(CredentialProfile {
                        name,
                        is_active: false,
                        info,
                    });
                }
            }
        }
    }

    // Sắp xếp: active first, sau đó theo tên
    profiles.sort_by(|a, b| {
        if a.is_active {
            std::cmp::Ordering::Less
        } else if b.is_active {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(profiles)
}

/// Lưu credentials hiện tại thành profile mới
/// Rename .credentials.json → .credentials-[name].json
#[tauri::command]
pub async fn save_current_as_profile(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let active_path = dir.join(".credentials.json");
    let target_path = dir.join(format!(".credentials-{}.json", name));

    if !active_path.exists() {
        return Err("No active credentials found (.credentials.json)".to_string());
    }

    if target_path.exists() {
        return Err(format!("Profile '{}' already exists", name));
    }

    // Copy (không rename, vì user vẫn cần credential active)
    std::fs::copy(&active_path, &target_path).map_err(|e| e.to_string())?;

    // Giữ permissions 600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Switch sang profile khác
/// 1. Rename .credentials.json → .credentials-[currentName].json (save current)
/// 2. Rename .credentials-[targetName].json → .credentials.json (activate target)
#[tauri::command]
pub async fn switch_credential_profile(
    app: tauri::AppHandle,
    current_name: String,
    target_name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let active_path = dir.join(".credentials.json");
    let current_backup = dir.join(format!(".credentials-{}.json", current_name));
    let target_path = dir.join(format!(".credentials-{}.json", target_name));

    // Validate target tồn tại
    if !target_path.exists() {
        return Err(format!("Profile '{}' not found", target_name));
    }

    // Step 1: Save current active → backup
    if active_path.exists() {
        // Nếu backup đã tồn tại, ghi đè (cập nhật token mới nhất)
        std::fs::rename(&active_path, &current_backup)
            .map_err(|e| format!("Failed to backup current credentials: {}", e))?;
    }

    // Step 2: Activate target → .credentials.json
    std::fs::rename(&target_path, &active_path)
        .map_err(|e| format!("Failed to activate target credentials: {}", e))?;

    Ok(())
}

/// Đổi tên profile
#[tauri::command]
pub async fn rename_credential_profile(
    app: tauri::AppHandle,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let old_path = dir.join(format!(".credentials-{}.json", old_name));
    let new_path = dir.join(format!(".credentials-{}.json", new_name));

    if !old_path.exists() {
        return Err(format!("Profile '{}' not found", old_name));
    }
    if new_path.exists() {
        return Err(format!("Profile '{}' already exists", new_name));
    }

    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Xóa profile đã lưu
#[tauri::command]
pub async fn delete_credential_profile(
    app: tauri::AppHandle,
    name: String,
) -> Result<(), String> {
    let dir = claude_dir(&app)?;
    let path = dir.join(format!(".credentials-{}.json", name));

    if !path.exists() {
        return Err(format!("Profile '{}' not found", name));
    }

    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(())
}

// ========== CLI State (giữ nguyên) ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeCliState {
    pub current_model: Option<String>,
    pub session_count: usize,
    pub env_file_exists: bool,
    pub active_keys: Vec<String>,
}

#[tauri::command]
pub async fn get_claude_cli_state(app: tauri::AppHandle) -> Result<ClaudeCliState, String> {
    let dir = claude_dir(&app)?;

    let current_model = read_settings_model(&dir);
    let session_count = count_history_sessions(&dir);

    let env_path = dir.join(".env");
    let env_file_exists = env_path.exists();
    let active_keys = if env_file_exists {
        read_env_key_names(&env_path)
    } else {
        vec![]
    };

    Ok(ClaudeCliState {
        current_model,
        session_count,
        env_file_exists,
        active_keys,
    })
}

fn read_settings_model(dir: &PathBuf) -> Option<String> {
    let path = dir.join("settings.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("model")?.as_str().map(|s| s.to_string())
}

fn count_history_sessions(dir: &PathBuf) -> usize {
    let path = dir.join("history.jsonl");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .count()
}

fn read_env_key_names(env_path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(env_path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.starts_with('#') && l.contains('='))
        .map(|l| l.split('=').next().unwrap_or("").trim().to_string())
        .filter(|k| !k.is_empty())
        .collect()
}
