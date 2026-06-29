use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::commands::session_usage_commands::parse_session_logs;
use crate::modules::usage::models::{
    DayUsage, ModelUsage, PriceStatus, SessionUsage, TokenBreakdown, UsageReport,
};
use crate::modules::usage::pricing::{load_price_table, ModelPrice};

/// Build the Claude Code usage + cost report over the last `range_days`
/// (0 = all time). Token data is reused from the session-log parser; cost is
/// derived from the LiteLLM price table.
pub async fn build_report(claude_dir: &Path, cache_dir: &Path, range_days: u32) -> UsageReport {
    // "Today" (1 day) buckets by hour from local midnight; longer ranges by day.
    let hourly = range_days == 1;
    let since = if hourly {
        start_of_today_utc()
    } else {
        since_for(range_days)
    };
    let summaries = parse_session_logs(&claude_dir.to_path_buf(), since);
    let prices = load_price_table(cache_dir).await;
    let have_prices = prices.status != PriceStatus::Hidden;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut total = TokenBreakdown::default();
    let mut total_cost = 0.0;
    let mut today_tokens = TokenBreakdown::default();
    let mut today_cost = 0.0;
    let mut daily: BTreeMap<String, (TokenBreakdown, f64)> = BTreeMap::new();
    let mut by_model: HashMap<String, (TokenBreakdown, f64)> = HashMap::new();
    let mut sessions: Vec<SessionUsage> = Vec::new();

    for s in &summaries {
        let tokens = TokenBreakdown {
            input: s.total_input_tokens,
            output: s.total_output_tokens,
            cache_read: s.total_cache_read,
            cache_creation: s.total_cache_write,
        };
        let cal_date = local_date(&s.started_at);
        // Chart bucket: hour-of-day in "today" mode, otherwise the calendar date.
        let bucket = if hourly {
            local_hour(&s.started_at)
        } else {
            cal_date.clone()
        };
        let cost = prices.lookup(&s.model).map(|p| cost_of(&tokens, &p));

        total.add(&tokens);
        total_cost += cost.unwrap_or(0.0);

        let day = daily.entry(bucket.clone()).or_default();
        day.0.add(&tokens);
        day.1 += cost.unwrap_or(0.0);

        let model = by_model.entry(s.model.clone()).or_default();
        model.0.add(&tokens);
        model.1 += cost.unwrap_or(0.0);

        if cal_date == today {
            today_tokens.add(&tokens);
            today_cost += cost.unwrap_or(0.0);
        }

        sessions.push(SessionUsage {
            id: s.session_id.clone(),
            date: bucket,
            model: s.model.clone(),
            project: s.project.clone(),
            tokens,
            cost_usd: cost,
        });
    }

    let daily_vec: Vec<DayUsage> = daily
        .into_iter()
        .map(|(date, (tokens, cost))| DayUsage {
            date,
            tokens,
            cost_usd: have_prices.then_some(cost),
        })
        .collect();

    let mut by_model_vec: Vec<ModelUsage> = by_model
        .into_iter()
        .map(|(model, (tokens, cost))| ModelUsage {
            model,
            tokens,
            cost_usd: have_prices.then_some(cost),
        })
        .collect();
    by_model_vec.sort_by(|a, b| total_tokens(&b.tokens).cmp(&total_tokens(&a.tokens)));

    // summaries are sorted oldest-first → reverse for newest-first, keep latest 30
    sessions.reverse();
    sessions.truncate(30);

    UsageReport {
        total,
        total_cost_usd: have_prices.then_some(total_cost),
        today: today_tokens,
        today_cost_usd: have_prices.then_some(today_cost),
        daily: daily_vec,
        by_model: by_model_vec,
        sessions,
        generated_at: chrono::Utc::now().to_rfc3339(),
        price_status: prices.status,
        price_updated_at: prices.updated_at,
    }
}

/// Local midnight of the current day, expressed in UTC.
fn start_of_today_utc() -> chrono::DateTime<chrono::Utc> {
    let now = chrono::Local::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| naive.and_local_timezone(chrono::Local).single())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now)
}

/// Local hour-of-day label, e.g. "08:00".
fn local_hour(rfc3339: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(rfc3339) {
        Ok(ts) => ts
            .with_timezone(&chrono::Local)
            .format("%H:00")
            .to_string(),
        Err(_) => "00:00".to_string(),
    }
}

fn since_for(range_days: u32) -> chrono::DateTime<chrono::Utc> {
    if range_days == 0 {
        chrono::DateTime::from_timestamp(0, 0).unwrap_or_else(chrono::Utc::now)
    } else {
        chrono::Utc::now() - chrono::Duration::days(range_days as i64)
    }
}

fn local_date(rfc3339: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(rfc3339) {
        Ok(ts) => ts
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string(),
        Err(_) => rfc3339.chars().take(10).collect(),
    }
}

fn cost_of(tokens: &TokenBreakdown, price: &ModelPrice) -> f64 {
    tokens.input as f64 * price.input
        + tokens.output as f64 * price.output
        + tokens.cache_read as f64 * price.cache_read
        + tokens.cache_creation as f64 * price.cache_creation
}

fn total_tokens(tokens: &TokenBreakdown) -> u64 {
    tokens.input + tokens.output + tokens.cache_read + tokens.cache_creation
}
