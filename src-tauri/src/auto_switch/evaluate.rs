use std::collections::HashMap;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::auto_switch::{store, COOLDOWN_MAX, COOLDOWN_MIN};
use crate::modules::providers::claude_cli::config::read_meta;
use crate::modules::quota::UsageLimits;
use crate::modules::shared::paths::claude_data_dir;

/// Key the quota worker uses for the credential source currently in use, as
/// opposed to the profile folders it also scans.
const ACTIVE_KEY: &str = "active";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SwitchPerformed {
    from: String,
    to: String,
    utilization: f64,
    claude_was_running: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Exhausted {
    profile: String,
    utilization: f64,
}

fn five_hour_utilization(limits: &UsageLimits) -> Option<f64> {
    limits.five_hour.as_ref()?.utilization
}

/// Desktop notification. Failures are swallowed on purpose: a denied permission
/// or a platform without notification support must never block a switch.
fn notify(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
}

/// Profile with the lowest five-hour utilization that is not the active one.
///
/// The quota worker's map holds the active profile TWICE: once under the literal
/// "active" key (the credential source in use) and once under its own name (from
/// the profiles-dir scan, which also contains the active profile's folder).
/// Both keys must be dropped, otherwise the rule would pick the active profile
/// as the target and "switch" it onto itself.
fn pick_fallback(usage: &HashMap<String, UsageLimits>, active_name: &str) -> Option<(String, f64)> {
    usage
        .iter()
        .filter(|(name, _)| name.as_str() != ACTIVE_KEY && name.as_str() != active_name)
        .filter_map(|(name, limits)| five_hour_utilization(limits).map(|u| (name.clone(), u)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// True while the last automatic switch is still inside the configured cooldown.
fn in_cooldown(last_auto_switch_at: Option<&str>, cooldown_minutes: u64) -> bool {
    let Some(last) = last_auto_switch_at else {
        return false;
    };
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(last) else {
        return false;
    };
    // Re-clamped here as well: a hand-edited file could hold a value that would
    // overflow the duration conversion.
    let minutes = cooldown_minutes.clamp(COOLDOWN_MIN, COOLDOWN_MAX) as i64;
    let elapsed = chrono::Local::now().signed_duration_since(parsed.with_timezone(&chrono::Local));
    elapsed < chrono::TimeDelta::minutes(minutes)
}

/// Switch the active profile away when it reached the configured threshold.
/// Called once per quota-worker cycle with that cycle's usage snapshot.
pub async fn evaluate_and_switch(app: &AppHandle, usage: &HashMap<String, UsageLimits>) {
    let cfg = store::load(app);
    if !cfg.enabled {
        return;
    }

    let Ok(data_dir) = claude_data_dir(app) else {
        return;
    };
    let Some(active_name) = read_meta(&data_dir).active_profile_name else {
        return;
    };

    // The active profile is reachable under its own name or under "active",
    // depending on where its credential lives.
    let Some(active_util) = usage
        .get(&active_name)
        .or_else(|| usage.get(ACTIVE_KEY))
        .and_then(five_hour_utilization)
    else {
        return;
    };

    if active_util < cfg.threshold {
        // Quota came back down: allow the exhausted notice to fire again later.
        store::set_exhausted_notified(app, false);
        return;
    }

    let fallback = pick_fallback(usage, &active_name).filter(|(_, util)| *util < cfg.threshold);
    let Some((target_name, _)) = fallback else {
        if !cfg.all_exhausted_notified {
            let _ = app.emit(
                "auto-switch-exhausted",
                Exhausted {
                    profile: active_name.clone(),
                    utilization: active_util,
                },
            );
            notify(
                app,
                "All profiles above threshold",
                &format!("{active_name} reached {active_util:.0}% and no profile is below it."),
            );
            store::record_exhausted(app, &active_name, active_util);
        }
        return;
    };

    // Silent on purpose: the worker ticks every few minutes, so logging or
    // notifying a cooldown hit would spam the user.
    if in_cooldown(cfg.last_auto_switch_at.as_deref(), cfg.cooldown_minutes) {
        return;
    }

    // Reuses the manual switch path so backup, reconcile, meta and tray stay
    // consistent — the tray is refreshed in there, do not refresh it again here.
    match crate::commands::config_commands::switch_credential_profile(
        app.clone(),
        target_name.clone(),
    )
    .await
    {
        Ok(result) => {
            store::record_switch(app, &active_name, &target_name, active_util);
            let _ = app.emit(
                "auto-switch-performed",
                SwitchPerformed {
                    from: active_name.clone(),
                    to: target_name.clone(),
                    utilization: active_util,
                    claude_was_running: result.claude_was_running,
                },
            );
            let mut body = format!("{active_name} reached {active_util:.0}%.");
            if result.claude_was_running {
                body.push_str(" Restart Claude Code to apply.");
            }
            notify(app, &format!("Switched to {target_name}"), &body);
        }
        // No cooldown is started on failure, so the next cycle retries.
        Err(e) => eprintln!("Auto switch to '{target_name}' failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::quota::UsageBucket;

    fn limits(utilization: Option<f64>) -> UsageLimits {
        UsageLimits {
            five_hour: Some(UsageBucket {
                utilization,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn skips_both_keys_of_the_active_profile() {
        let mut usage = HashMap::new();
        // Same profile twice, as the quota worker reports it.
        usage.insert("active".to_string(), limits(Some(95.0)));
        usage.insert("me@example.com".to_string(), limits(Some(95.0)));
        usage.insert("other@example.com".to_string(), limits(Some(40.0)));

        let picked = pick_fallback(&usage, "me@example.com");
        assert_eq!(picked, Some(("other@example.com".to_string(), 40.0)));
    }

    #[test]
    fn picks_the_lowest_utilization() {
        let mut usage = HashMap::new();
        usage.insert("a@example.com".to_string(), limits(Some(70.0)));
        usage.insert("b@example.com".to_string(), limits(Some(12.5)));
        usage.insert("c@example.com".to_string(), limits(Some(30.0)));

        let picked = pick_fallback(&usage, "active-profile");
        assert_eq!(picked, Some(("b@example.com".to_string(), 12.5)));
    }

    #[test]
    fn ignores_profiles_without_a_five_hour_reading() {
        let mut usage = HashMap::new();
        usage.insert("a@example.com".to_string(), limits(None));
        usage.insert("b@example.com".to_string(), UsageLimits::default());

        assert!(pick_fallback(&usage, "active-profile").is_none());
    }

    #[test]
    fn no_fallback_when_only_the_active_profile_is_known() {
        let mut usage = HashMap::new();
        usage.insert("active".to_string(), limits(Some(99.0)));
        usage.insert("me@example.com".to_string(), limits(Some(99.0)));

        assert!(pick_fallback(&usage, "me@example.com").is_none());
    }

    #[test]
    fn cooldown_is_open_without_a_previous_switch() {
        assert!(!in_cooldown(None, 5));
    }

    #[test]
    fn cooldown_blocks_a_recent_switch_and_opens_later() {
        let now = chrono::Local::now();
        let recent = (now - chrono::TimeDelta::minutes(2)).to_rfc3339();
        let old = (now - chrono::TimeDelta::minutes(30)).to_rfc3339();

        assert!(in_cooldown(Some(&recent), 5));
        assert!(!in_cooldown(Some(&old), 5));
    }

    #[test]
    fn cooldown_is_open_on_an_unparsable_stamp() {
        assert!(!in_cooldown(Some("not-a-date"), 5));
    }
}
