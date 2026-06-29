// One-time migration of data from the pre-rebrand layout.
//
// The app was renamed (claude-tools → agent-switch-tools) and its data root moved
// from `~/.claude/.claude-tools` to `~/.agent-switch-tools/claude`. Installs that
// upgrade across the rename leave account profiles, switch history, and device
// identity stranded in the old location. This copies anything the new layout is
// missing — it never overwrites newer data — and drops a marker so it runs once.

use std::path::{Path, PathBuf};

use crate::modules::providers::claude_cli::config::{read_meta, write_meta};

const MARKER: &str = ".legacy-migrated";

/// Entry point: best-effort, never fails the app startup.
pub fn migrate_legacy_data(home: &Path) {
    if let Err(e) = try_migrate(home) {
        eprintln!("[migrate] legacy data migration skipped: {e}");
    }
}

fn try_migrate(home: &Path) -> Result<(), String> {
    let old_root = home.join(".claude").join(".claude-tools");
    if !old_root.exists() {
        return Ok(());
    }
    let new_root = home.join(".agent-switch-tools");
    let marker = new_root.join(MARKER);
    if marker.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(&new_root).map_err(|e| e.to_string())?;

    migrate_profiles(&old_root, &new_root)?;
    migrate_meta(&old_root, &new_root);
    migrate_device(&old_root, &new_root);

    std::fs::write(&marker, "1").map_err(|e| e.to_string())?;
    Ok(())
}

/// Copy each legacy Claude profile the new layout doesn't already have.
/// Profiles that exist in the new location are newer and left untouched.
fn migrate_profiles(old_root: &Path, new_root: &Path) -> Result<(), String> {
    let old_profiles = old_root.join("profiles");
    if !old_profiles.is_dir() {
        return Ok(());
    }
    let new_profiles = new_root.join("claude").join("profiles");
    std::fs::create_dir_all(&new_profiles).map_err(|e| e.to_string())?;

    for entry in std::fs::read_dir(&old_profiles).map_err(|e| e.to_string())?.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let dest = new_profiles.join(entry.file_name());
        if dest.exists() {
            continue; // keep newer data
        }
        copy_dir(&entry.path(), &dest)?;
    }
    Ok(())
}

/// Backfill usage history from the legacy meta, and adopt its active profile only
/// when the new meta has none yet. Never clobbers a value the new layout already set.
fn migrate_meta(old_root: &Path, new_root: &Path) {
    let old_dir: PathBuf = old_root.to_path_buf();
    if !old_dir.join("meta.json").exists() {
        return;
    }
    let new_dir: PathBuf = new_root.join("claude");

    let old = read_meta(&old_dir);
    let mut new = read_meta(&new_dir);

    for (name, usage) in old.usage_history {
        new.usage_history.entry(name).or_insert(usage);
    }
    if new.active_profile_name.is_none() {
        new.active_profile_name = old.active_profile_name;
        if new.last_switched_at.is_none() {
            new.last_switched_at = old.last_switched_at;
        }
    }
    let _ = write_meta(&new_dir, &new);
}

fn migrate_device(old_root: &Path, new_root: &Path) {
    let old_device = old_root.join("device.json");
    let new_device = new_root.join("device.json");
    if old_device.exists() && !new_device.exists() {
        let _ = std::fs::copy(&old_device, &new_device);
    }
}

fn copy_dir(src: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        let target = dest.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| e.to_string())?;
            set_600(&target);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn set_600(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}
#[cfg(not(unix))]
fn set_600(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn seed_old(home: &Path) {
        let old = home.join(".claude").join(".claude-tools");
        write(&old.join("profiles/anhtct@inet.vn/credentials.json"), "OLD-anhtct");
        write(&old.join("profiles/trihd@inet.vn/credentials.json"), "OLD-trihd");
        write(&old.join("device.json"), "old-device");
        write(
            &old.join("meta.json"),
            r#"{"activeProfileName":"trihd@inet.vn","lastSwitchedAt":"t0","usageHistory":{"trihd@inet.vn":{"totalActiveMinutes":10.0}}}"#,
        );
    }

    #[test]
    fn copies_missing_profile_but_keeps_newer_one() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        seed_old(home);
        // New layout already has a (newer) anhtct profile.
        write(
            &home.join(".agent-switch-tools/claude/profiles/anhtct@inet.vn/credentials.json"),
            "NEW-anhtct",
        );

        migrate_legacy_data(home);

        let new_profiles = home.join(".agent-switch-tools/claude/profiles");
        // Newer profile preserved, missing one backfilled.
        assert_eq!(
            std::fs::read_to_string(new_profiles.join("anhtct@inet.vn/credentials.json")).unwrap(),
            "NEW-anhtct"
        );
        assert_eq!(
            std::fs::read_to_string(new_profiles.join("trihd@inet.vn/credentials.json")).unwrap(),
            "OLD-trihd"
        );
        // device.json backfilled, marker written.
        assert!(home.join(".agent-switch-tools/device.json").exists());
        assert!(home.join(".agent-switch-tools").join(MARKER).exists());
    }

    #[test]
    fn backfills_usage_history_into_existing_meta() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        seed_old(home);
        // New meta already active = anhtct; must not be overwritten.
        write(
            &home.join(".agent-switch-tools/claude/meta.json"),
            r#"{"activeProfileName":"anhtct@inet.vn","usageHistory":{}}"#,
        );

        migrate_legacy_data(home);

        let meta = read_meta(&home.join(".agent-switch-tools/claude"));
        assert_eq!(meta.active_profile_name.as_deref(), Some("anhtct@inet.vn"));
        assert!(meta.usage_history.contains_key("trihd@inet.vn"));
    }

    #[test]
    fn runs_only_once() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        seed_old(home);
        migrate_legacy_data(home);

        // User later deletes a profile; a second run must not resurrect it.
        let trihd = home.join(".agent-switch-tools/claude/profiles/trihd@inet.vn");
        std::fs::remove_dir_all(&trihd).unwrap();
        migrate_legacy_data(home);
        assert!(!trihd.exists());
    }

    #[test]
    fn no_old_dir_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        migrate_legacy_data(tmp.path());
        assert!(!tmp.path().join(".agent-switch-tools").join(MARKER).exists());
    }
}
