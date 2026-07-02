// Regression tests for the Claude login drift fix.
//
// Bug: when the user logged into a different Claude account outside the app
// (`claude /login`), the next save/switch operation backed up the new credentials
// into the previous email's profile folder, destroying the old profile.
//
// Fix: reconcile_active_profile() syncs meta with `~/.claude.json` before any
// backup decision and saves the live credentials into a folder named after the
// actual email — preserving any existing folder for the old email untouched.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tauri_app_lib::modules::providers::claude_cli::reconcile::{
    reconcile_active_profile, validate_email_as_folder,
};

struct TestEnv {
    _tmp: TempDir,
    home: PathBuf,
    claude: PathBuf,
    profs_dir: PathBuf,
    claude_data: PathBuf,
}

fn setup() -> TestEnv {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();
    let claude = home.join(".claude");
    let claude_data = home.join(".agent-switch-tools").join("claude");
    let profs_dir = claude_data.join("profiles");
    fs::create_dir_all(&claude).unwrap();
    fs::create_dir_all(&profs_dir).unwrap();
    TestEnv {
        _tmp: tmp,
        home,
        claude,
        profs_dir,
        claude_data,
    }
}

fn write_claude_json(home: &PathBuf, email: &str) {
    // Claude Code writes the account identity into `oauthAccount` — the only
    // key reconcile reads (the legacy `claudeAiOauth` cache is ignored).
    let json = serde_json::json!({
        "oauthAccount": {
            "emailAddress": email,
        }
    });
    fs::write(home.join(".claude.json"), json.to_string()).unwrap();
}

fn write_active_credentials(claude: &PathBuf, content: &str) {
    fs::write(claude.join(".credentials.json"), content).unwrap();
}

fn write_meta(claude_data: &PathBuf, active_name: Option<&str>) {
    fs::create_dir_all(claude_data).unwrap();
    let meta = match active_name {
        Some(n) => serde_json::json!({ "activeProfileName": n }),
        None => serde_json::json!({}),
    };
    fs::write(claude_data.join("meta.json"), meta.to_string()).unwrap();
}

#[test]
fn no_credentials_returns_none() {
    let env = setup();
    let result =
        reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (None, false));
}

#[test]
fn synced_meta_no_drift() {
    let env = setup();
    write_claude_json(&env.home, "alice@example.com");
    write_active_credentials(&env.claude, r#"{"claudeAiOauth":{"accessToken":"a"}}"#);
    write_meta(&env.claude_data, Some("alice@example.com"));

    let result =
        reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (Some("alice@example.com".into()), false));

    // Even without drift the active account's backup is refreshed on every
    // reconcile, so the snapshot kept for it is always the freshest one.
    let backup = fs::read_to_string(
        env.profs_dir
            .join("alice@example.com")
            .join("credentials.json"),
    )
    .unwrap();
    assert!(
        backup.contains(r#""accessToken":"a""#),
        "backup must mirror the live credentials"
    );
}

#[test]
fn drift_saves_new_email_preserves_old_folder() {
    let env = setup();
    // Active credentials and ~/.claude.json now point at bob, but meta still says alice.
    write_claude_json(&env.home, "bob@example.com");
    write_active_credentials(
        &env.claude,
        r#"{"claudeAiOauth":{"accessToken":"bob_token"}}"#,
    );
    write_meta(&env.claude_data, Some("alice@example.com"));

    // Pre-existing alice folder with old credentials — must NOT be overwritten.
    let alice_dir = env.profs_dir.join("alice@example.com");
    fs::create_dir_all(&alice_dir).unwrap();
    fs::write(
        alice_dir.join("credentials.json"),
        r#"{"claudeAiOauth":{"accessToken":"alice_token"}}"#,
    )
    .unwrap();

    let result =
        reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (Some("bob@example.com".into()), true));

    // Alice folder must remain untouched.
    let alice_creds = fs::read_to_string(alice_dir.join("credentials.json")).unwrap();
    assert!(
        alice_creds.contains("alice_token"),
        "alice credentials must be preserved"
    );

    // Bob folder must be created with the live credentials.
    let bob_dir = env.profs_dir.join("bob@example.com");
    assert!(bob_dir.exists(), "bob folder must be created");
    let bob_creds = fs::read_to_string(bob_dir.join("credentials.json")).unwrap();
    assert!(
        bob_creds.contains("bob_token"),
        "bob credentials must match live credentials"
    );

    // oauth.json must be saved next to credentials.json.
    assert!(
        bob_dir.join("oauth.json").exists(),
        "oauth.json must be persisted"
    );

    // Meta must be updated to bob.
    let meta_content = fs::read_to_string(env.claude_data.join("meta.json")).unwrap();
    assert!(
        meta_content.contains("bob@example.com"),
        "meta must be updated to bob"
    );
}

#[test]
fn missing_email_returns_none() {
    let env = setup();
    write_active_credentials(&env.claude, "{}");
    fs::write(
        env.home.join(".claude.json"),
        r#"{"oauthAccount":{}}"#,
    )
    .unwrap();

    let result =
        reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (None, false));
}

#[test]
fn empty_meta_first_save_no_drift_event() {
    // Fresh install: meta is empty, but credentials exist with email.
    // The reconcile should treat this as drift (cached "" != actual email),
    // so the credentials get saved into the email folder right away.
    let env = setup();
    write_claude_json(&env.home, "carol@example.com");
    write_active_credentials(
        &env.claude,
        r#"{"claudeAiOauth":{"accessToken":"carol_token"}}"#,
    );
    // No meta.json written.

    let result =
        reconcile_active_profile(&env.home, &env.claude, &env.profs_dir, &env.claude_data).unwrap();
    assert_eq!(result, (Some("carol@example.com".into()), true));
    assert!(env
        .profs_dir
        .join("carol@example.com")
        .join("credentials.json")
        .exists());
}

// ---------- validate_email_as_folder ----------

#[test]
fn validate_email_rejects_empty() {
    assert!(validate_email_as_folder("").is_err());
}

#[test]
fn validate_email_rejects_path_traversal() {
    assert!(validate_email_as_folder("../etc/passwd").is_err());
    assert!(validate_email_as_folder("a..b@x.com").is_err());
}

#[test]
fn validate_email_rejects_slashes() {
    assert!(validate_email_as_folder("a/b@x.com").is_err());
    assert!(validate_email_as_folder("a\\b@x.com").is_err());
}

#[test]
fn validate_email_rejects_hidden() {
    assert!(validate_email_as_folder(".hidden@x.com").is_err());
}

#[test]
fn validate_email_accepts_valid() {
    assert!(validate_email_as_folder("alice@example.com").is_ok());
    assert!(validate_email_as_folder("alice+tag@example.co.uk").is_ok());
    assert!(validate_email_as_folder("user.name-1@sub.domain.io").is_ok());
}
