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
        // No drift — but Claude Code rotates tokens while an account is active, and an
        // external login overwrites `.credentials.json` before we can react. Refreshing
        // the backup on every reconcile guarantees the snapshot we keep of this account
        // is the freshest one available when a new login eventually replaces it.
        refresh_active_backup(&active_path, profs_dir, &actual_email, &oauth);
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

/// Best-effort sync of the active account's saved copy with the live credentials
/// and identity. Errors are swallowed: a failed refresh only means a slightly
/// staler backup, never a broken listing.
fn refresh_active_backup(
    active_path: &PathBuf,
    profs_dir: &PathBuf,
    email: &str,
    oauth: &auth::OAuthAccount,
) {
    let Ok(prof_dir) = profile_dir(profs_dir, email) else {
        return;
    };

    let live = std::fs::read(active_path).unwrap_or_default();
    if !live.is_empty() {
        let backup_path = prof_dir.join("credentials.json");
        let stored = std::fs::read(&backup_path).unwrap_or_default();
        if live != stored && std::fs::write(&backup_path, &live).is_ok() {
            config::set_file_600(&backup_path);
        }
    }

    let new_identity = serde_json::to_string_pretty(oauth).unwrap_or_default();
    let stored_identity =
        std::fs::read_to_string(prof_dir.join("oauth.json")).unwrap_or_default();
    if !new_identity.is_empty() && new_identity != stored_identity {
        let _ = auth::write_saved_oauth(profs_dir, email, oauth);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Env {
        _tmp: tempfile::TempDir,
        home: PathBuf,
        claude: PathBuf,
        profs: PathBuf,
        data: PathBuf,
    }

    /// Sandbox mirroring the real layout: ~/.claude.json, ~/.claude/.credentials.json,
    /// app data dir with meta.json and profiles/.
    fn setup(actual_email: &str, cached_email: &str) -> Env {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().to_path_buf();
        let claude = home.join(".claude");
        let data = home.join("appdata");
        let profs = data.join("profiles");
        std::fs::create_dir_all(&claude).unwrap();
        std::fs::create_dir_all(&profs).unwrap();

        std::fs::write(
            home.join(".claude.json"),
            format!(r#"{{ "oauthAccount": {{ "emailAddress": "{}" }} }}"#, actual_email),
        )
        .unwrap();
        std::fs::write(claude.join(".credentials.json"), r#"{"claudeAiOauth":{"accessToken":"new-token"}}"#).unwrap();

        let meta = config::ManagerMeta {
            active_profile_name: Some(cached_email.to_string()),
            ..Default::default()
        };
        config::write_meta(&data, &meta).unwrap();

        Env { _tmp: tmp, home, claude, profs, data }
    }

    /// The reported bug: user logs into a new account outside the app. Reconcile must
    /// detect the drift, snapshot the new account into its own folder, keep the old
    /// account's folder untouched, and point meta at the new email.
    #[test]
    fn external_login_saves_new_account_and_preserves_old() {
        let env = setup("chuongdt@test.vn", "anhtct@test.vn");

        // Pre-existing backup of the old account must survive.
        let old_dir = env.profs.join("anhtct@test.vn");
        std::fs::create_dir_all(&old_dir).unwrap();
        std::fs::write(old_dir.join("credentials.json"), r#"{"claudeAiOauth":{"accessToken":"old-token"}}"#).unwrap();

        let (actual, drift) =
            reconcile_active_profile(&env.home, &env.claude, &env.profs, &env.data).unwrap();

        assert_eq!(actual.as_deref(), Some("chuongdt@test.vn"));
        assert!(drift, "external login must be detected as drift");

        let new_cred = env.profs.join("chuongdt@test.vn").join("credentials.json");
        assert!(new_cred.exists(), "new account must be snapshotted");
        assert!(
            std::fs::read_to_string(env.profs.join("anhtct@test.vn").join("credentials.json"))
                .unwrap()
                .contains("old-token"),
            "old account backup must not be overwritten"
        );
        assert_eq!(
            config::read_meta(&env.data).active_profile_name.as_deref(),
            Some("chuongdt@test.vn")
        );
    }

    /// No drift, but the active account's backup must still be kept fresh: Claude Code
    /// rotates tokens over time, and this snapshot is all that survives the next
    /// external login.
    #[test]
    fn matching_email_refreshes_stale_backup_without_drift() {
        let env = setup("anhtct@test.vn", "anhtct@test.vn");

        let prof_dir = env.profs.join("anhtct@test.vn");
        std::fs::create_dir_all(&prof_dir).unwrap();
        std::fs::write(prof_dir.join("credentials.json"), r#"{"claudeAiOauth":{"accessToken":"rotated-out-token"}}"#).unwrap();

        let (actual, drift) =
            reconcile_active_profile(&env.home, &env.claude, &env.profs, &env.data).unwrap();

        assert_eq!(actual.as_deref(), Some("anhtct@test.vn"));
        assert!(!drift);
        assert!(
            std::fs::read_to_string(prof_dir.join("credentials.json"))
                .unwrap()
                .contains("new-token"),
            "backup must be synced with the live credentials"
        );
        assert!(prof_dir.join("oauth.json").exists(), "identity must be saved alongside");
    }
}
