use std::fs;

fn main() {
    // Load local .env files so `option_env!` picks up OAuth client credentials at compile time.
    // CI provides these via the process environment instead (these calls are then no-ops).
    load_env_file("../.env");
    load_env_file(".env");
    tauri_build::build()
}

/// Parse a simple KEY=VALUE `.env` file and forward each entry to the crate's compile-time
/// environment via `cargo:rustc-env` (only when not already set in the process environment).
fn load_env_file(path: &str) {
    println!("cargo:rerun-if-changed={}", path);
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if key.is_empty() || value.is_empty() {
            continue;
        }
        if std::env::var(key).is_err() {
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
}
