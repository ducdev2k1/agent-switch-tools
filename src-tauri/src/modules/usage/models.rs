use serde::Serialize;

/// Token counts split by billing category. Values come straight from the
/// `message.usage` blocks in Claude Code session logs (real, not estimated).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBreakdown {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_creation: u64,
}

impl TokenBreakdown {
    pub fn add(&mut self, other: &TokenBreakdown) {
        self.input += other.input;
        self.output += other.output;
        self.cache_read += other.cache_read;
        self.cache_creation += other.cache_creation;
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayUsage {
    pub date: String,
    pub tokens: TokenBreakdown,
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    pub tokens: TokenBreakdown,
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUsage {
    pub id: String,
    pub date: String,
    pub model: String,
    pub project: String,
    pub tokens: TokenBreakdown,
    pub cost_usd: Option<f64>,
}

/// Whether the report's cost figures came from live, cached, or no pricing data.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PriceStatus {
    Live,
    Saved,
    Hidden,
}

/// Full usage + cost report for Claude Code over the requested date range.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub total: TokenBreakdown,
    pub total_cost_usd: Option<f64>,
    pub today: TokenBreakdown,
    pub today_cost_usd: Option<f64>,
    pub daily: Vec<DayUsage>,
    pub by_model: Vec<ModelUsage>,
    pub sessions: Vec<SessionUsage>,
    pub generated_at: String,
    pub price_status: PriceStatus,
    pub price_updated_at: Option<String>,
}
