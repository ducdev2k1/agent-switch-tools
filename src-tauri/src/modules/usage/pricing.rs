use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::modules::usage::models::PriceStatus;

const LITELLM_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";
const PRICE_TTL_SECS: i64 = 24 * 60 * 60;
const CACHE_FILE: &str = "litellm_prices.json";

/// Per-token prices for one model (USD).
#[derive(Debug, Clone, Copy, Default)]
pub struct ModelPrice {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_creation: f64,
}

#[derive(Serialize, Deserialize)]
struct PriceCache {
    updated_at: String,
    data: serde_json::Value,
}

/// Resolved price table plus provenance for UI badges.
pub struct PriceTable {
    models: HashMap<String, ModelPrice>,
    pub status: PriceStatus,
    pub updated_at: Option<String>,
}

impl PriceTable {
    /// Best-effort price lookup: exact, then without provider prefix, then
    /// without a trailing `-YYYYMMDD` date suffix.
    pub fn lookup(&self, model: &str) -> Option<ModelPrice> {
        if self.models.is_empty() {
            return None;
        }
        if let Some(p) = self.models.get(model) {
            return Some(*p);
        }
        let no_prefix = model.rsplit('/').next().unwrap_or(model);
        if let Some(p) = self.models.get(no_prefix) {
            return Some(*p);
        }
        self.models.get(strip_date_suffix(no_prefix)).copied()
    }
}

/// Load prices, preferring a disk cache fresher than 24h before hitting the
/// network. Falls back to a stale cache (then to "hidden") when offline.
pub async fn load_price_table(cache_dir: &Path) -> PriceTable {
    let cache_path = cache_dir.join(CACHE_FILE);

    if let Some(cache) = read_cache(&cache_path) {
        if !is_stale(&cache.updated_at) {
            let models = parse_models(&cache.data);
            return table(models, PriceStatus::Saved, Some(cache.updated_at));
        }
    }

    match fetch_remote().await {
        Some(data) => {
            let now = chrono::Utc::now().to_rfc3339();
            let _ = write_cache(&cache_path, &now, &data);
            let models = parse_models(&data);
            table(models, PriceStatus::Live, Some(now))
        }
        None => match read_cache(&cache_path) {
            Some(cache) => {
                let models = parse_models(&cache.data);
                table(models, PriceStatus::Saved, Some(cache.updated_at))
            }
            None => PriceTable {
                models: HashMap::new(),
                status: PriceStatus::Hidden,
                updated_at: None,
            },
        },
    }
}

/// Wrap a model map, downgrading to Hidden when no prices were parsed.
fn table(models: HashMap<String, ModelPrice>, ok: PriceStatus, updated_at: Option<String>) -> PriceTable {
    let status = if models.is_empty() { PriceStatus::Hidden } else { ok };
    PriceTable { models, status, updated_at }
}

async fn fetch_remote() -> Option<serde_json::Value> {
    let res = crate::modules::shared::http::CLIENT
        .get(LITELLM_URL)
        .send()
        .await
        .ok()?;
    if !res.status().is_success() {
        return None;
    }
    res.json().await.ok()
}

fn read_cache(path: &Path) -> Option<PriceCache> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_cache(path: &Path, updated_at: &str, data: &serde_json::Value) -> Result<(), String> {
    let cache = PriceCache {
        updated_at: updated_at.to_string(),
        data: data.clone(),
    };
    let json = serde_json::to_string(&cache).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

fn is_stale(updated_at: &str) -> bool {
    match chrono::DateTime::parse_from_rfc3339(updated_at) {
        Ok(ts) => {
            let age = chrono::Utc::now().signed_duration_since(ts.with_timezone(&chrono::Utc));
            age.num_seconds() > PRICE_TTL_SECS
        }
        Err(_) => true,
    }
}

fn parse_models(data: &serde_json::Value) -> HashMap<String, ModelPrice> {
    let mut map = HashMap::new();
    let Some(obj) = data.as_object() else {
        return map;
    };
    for (name, entry) in obj {
        let Some(input) = entry.get("input_cost_per_token").and_then(|v| v.as_f64()) else {
            continue;
        };
        map.insert(
            name.clone(),
            ModelPrice {
                input,
                output: field(entry, "output_cost_per_token"),
                cache_read: field(entry, "cache_read_input_token_cost"),
                cache_creation: field(entry, "cache_creation_input_token_cost"),
            },
        );
    }
    map
}

fn field(entry: &serde_json::Value, key: &str) -> f64 {
    entry.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

/// Trim a trailing `-YYYYMMDD` release-date suffix from a model id.
fn strip_date_suffix(model: &str) -> &str {
    if let Some(idx) = model.rfind('-') {
        let suffix = &model[idx + 1..];
        if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_digit()) {
            return &model[..idx];
        }
    }
    model
}
