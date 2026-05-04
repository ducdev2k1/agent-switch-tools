// Reconcile active Claude profile with the current `~/.claude.json` source of truth.
//
// Background:
// `meta.active_profile_name` is a cache. When the user runs `claude /login` outside the app,
// `~/.claude/.credentials.json` and `~/.claude.json` get rewritten with new account data,
// but `meta.active_profile_name` still points at the previous email. If the next operation
// uses the cached value to decide where to back up, the new credentials get written into the
// previous account's folder — destroying the old profile.
//
// This module syncs the cache with reality before any backup/switch decision: if the meta
// email differs from the actual `oauthAccount.emailAddress`, we save the current credentials
// into a folder named after the actual email (preserving any existing folder for the old
// email untouched) and update meta.

use std::path::PathBuf;

use crate::modules::providers::claude_cli::{auth, config};
use crate::modules::shared::paths::profile_dir;

/// Reject email values that would resolve to an unsafe folder name.
/// Folder names live under `profiles_dir`, so block path-traversal characters
/// plus Windows-reserved characters and control bytes — `~/.claude.json` is
/// user-writable so we can't trust the email field to be RFC-compliant.
pub fn validate_email_as_folder(email: &str) -> Result<(), String> {
    if email.is_empty() || email.starts_with('.') || email.contains("..") {
        return Err(format!("Invalid email for folder name: '{}'", email));
    }
    let forbidden = ['/', '\\', ':', '*', '?', '<', '>', '|', '"', '\0'];
    if email.chars().any(|c| forbidden.contains(&c) || c.is_control()) {
        return Err(format!("Invalid email for folder name: '{}'", email));
    }
    Ok(())
}

/// Sync meta.active_profile_name with the actual email in `~/.claude.json`.
///
/// Returns `(actual_email, drift_detected)`:
/// - `actual_email` is None when there are no active credentials or no email field.
/// - `drift_detected` is true when meta cache disagreed with reality and we auto-saved
///   the active credentials into the new email's folder.
///
/// On drift: copies `.credentials.json` to `profiles/{actual_email}/credentials.json`,
/// writes oauth.json next to it, updates meta.active_profile_name. Existing folders for
/// other emails (including the previously-cached one) are not touched.
pub fn reconcile_active_profile(
    home: &PathBuf,
    claude: &PathBuf,
    profs_dir: &PathBuf,
    claude_data: &PathBuf,
) -> Result<(Option<String>, bool), String> {
    let active_path = claude.join(".credentials.json");
    if !active_path.exists() {
        return Ok((None, false));
    }

    let oauth = match auth::read_oauth_from_claude_json(home) {
        Some(o) => o,
        None => return Ok((None, false)),
    };
    let actual_email = match oauth.email_address.as_ref() {
        Some(e) if !e.is_empty() => e.clone(),
        _ => return Ok((None, false)),
    };

    validate_email_as_folder(&actual_email)?;

    let mut meta = config::read_meta(claude_data);
    let cached_email = meta.active_profile_name.clone().unwrap_or_default();

    if cached_email == actual_email {
        return Ok((Some(actual_email), false));
    }

    let prof_dir = profile_dir(profs_dir, &actual_email)?;
    let backup_path = prof_dir.join("credentials.json");
    std::fs::copy(&active_path, &backup_path)
        .map_err(|e| format!("Reconcile: failed to save active credentials: {}", e))?;
    config::set_file_600(&backup_path);

    auth::write_saved_oauth(profs_dir, &actual_email, &oauth)?;

    meta.active_profile_name = Some(actual_email.clone());
    meta.last_switched_at = Some(chrono::Utc::now().to_rfc3339());
    config::write_meta(claude_data, &meta)?;

    #[cfg(debug_assertions)]
    eprintln!(
        "[reconcile] External login detected: cached={:?} actual={}",
        cached_email, actual_email
    );
    #[cfg(not(debug_assertions))]
    let _ = (&cached_email, &actual_email);

    Ok((Some(actual_email), true))
}
