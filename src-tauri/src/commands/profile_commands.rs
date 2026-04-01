use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub anthropic_api_key: String,
    pub gemini_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub model: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn get_profiles(app: tauri::AppHandle) -> Result<Vec<Profile>, String> {
    let store = app.store("profiles.json").map_err(|e| e.to_string())?;
    let profiles: Vec<Profile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    Ok(profiles)
}

#[tauri::command]
pub async fn save_profile(
    app: tauri::AppHandle,
    profile: Profile,
) -> Result<Profile, String> {
    let store = app.store("profiles.json").map_err(|e| e.to_string())?;
    let mut profiles: Vec<Profile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let now = chrono::Utc::now().to_rfc3339();
    let mut p = profile.clone();

    if p.id.is_empty() {
        // Tạo profile mới với UUID
        p.id = Uuid::new_v4().to_string();
        p.created_at = now.clone();
        p.updated_at = now;
        profiles.push(p.clone());
    } else {
        // Cập nhật profile hiện có
        p.updated_at = now;
        if let Some(existing) = profiles.iter_mut().find(|x| x.id == p.id) {
            *existing = p.clone();
        }
    }

    store.set("profiles", serde_json::to_value(&profiles).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(p)
}

#[tauri::command]
pub async fn delete_profile(
    app: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let store = app.store("profiles.json").map_err(|e| e.to_string())?;
    let mut profiles: Vec<Profile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    profiles.retain(|p| p.id != id);
    store.set("profiles", serde_json::to_value(&profiles).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_active_profile(
    app: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let store = app.store("profiles.json").map_err(|e| e.to_string())?;
    let mut profiles: Vec<Profile> = store
        .get("profiles")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Đặt tất cả profile về inactive, chỉ activate profile được chọn
    for p in profiles.iter_mut() {
        p.is_active = p.id == id;
    }

    store.set("profiles", serde_json::to_value(&profiles).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
