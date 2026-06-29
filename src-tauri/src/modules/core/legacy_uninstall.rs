// Best-effort removal of the pre-rebrand desktop app and its leftovers.
//
// Linux and Windows remove the old "Claude Tools" install through their package
// layer (deb `conflicts`/`replaces`, NSIS pre-install hook). macOS has no such
// layer for a *renamed* bundle, so the old app, its LaunchAgent login item, and
// its WebView caches are cleaned up here at startup.
//
// The cleanup logic is not `cfg`-gated (so it always type-checks), but only runs
// on macOS — every other platform returns immediately.

use std::path::{Path, PathBuf};

/// Old macOS bundle name and the brand fragment that identifies legacy artifacts.
const OLD_APP: &str = "Claude Tools.app";
const OLD_FRAGMENT: &str = "claude-tools";

pub fn remove_legacy_app(home: &Path) {
    if !cfg!(target_os = "macos") {
        return; // handled by the package manager / installer hook
    }
    remove_app_bundles(home);
    remove_login_items(home);
    remove_webview_leftovers(home);
}

/// Delete the old `.app`, but only when its Info.plist confirms the legacy bundle
/// id — so a folder that merely shares the name is never touched.
fn remove_app_bundles(home: &Path) {
    let candidates = [
        PathBuf::from("/Applications").join(OLD_APP),
        home.join("Applications").join(OLD_APP),
    ];
    for app in candidates {
        if !app.is_dir() {
            continue;
        }
        let plist = app.join("Contents").join("Info.plist");
        let is_legacy = std::fs::read_to_string(&plist)
            .map(|c| c.contains(OLD_FRAGMENT))
            .unwrap_or(false);
        if is_legacy {
            if let Err(e) = std::fs::remove_dir_all(&app) {
                eprintln!("[uninstall] could not remove {}: {e}", app.display());
            }
        }
    }
}

/// Unregister and delete LaunchAgent plists left by the old autostart entry,
/// otherwise launchd keeps trying to spawn a binary that no longer exists.
fn remove_login_items(home: &Path) {
    let agents = home.join("Library").join("LaunchAgents");
    for name in dir_names(&agents) {
        if !name.contains(OLD_FRAGMENT) {
            continue;
        }
        let plist = agents.join(&name);
        let label = name.trim_end_matches(".plist");
        let _ = std::process::Command::new("launchctl")
            .arg("unload")
            .arg("-w")
            .arg(&plist)
            .output();
        let _ = std::process::Command::new("launchctl")
            .arg("remove")
            .arg(label)
            .output();
        let _ = std::fs::remove_file(&plist);
    }
}

/// Remove orphaned WebView caches/state for the old bundle id. The app's real
/// data lives in `~/.agent-switch-tools` (migrated separately), so this only
/// clears OS-managed per-app storage keyed by the legacy identifier.
fn remove_webview_leftovers(home: &Path) {
    let lib = home.join("Library");
    let bases = [
        lib.join("Application Support"),
        lib.join("Caches"),
        lib.join("WebKit"),
    ];
    for base in bases {
        for name in dir_names(&base) {
            if name.contains(OLD_FRAGMENT) {
                let _ = std::fs::remove_dir_all(base.join(name));
            }
        }
    }
}

fn dir_names(dir: &Path) -> Vec<String> {
    match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    }
}
