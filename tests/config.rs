use cosmic_applet_opencode_quota::config::{auth_json_path, default_cache_path, resolve_api_key};
use serde_json::json;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

// Env-Variablen sind prozessglobal; die Tests, die OPENCODE_API_KEY setzen,
// müssen serialisiert laufen, um Races zu vermeiden.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn write_auth(dir: &tempfile::TempDir, obj: serde_json::Value) -> std::path::PathBuf {
    let p = dir.path().join("auth.json");
    fs::write(&p, serde_json::to_vec(&obj).unwrap()).unwrap();
    p
}

#[test]
fn resolve_key_from_auth_file() {
    let dir = tempdir().unwrap();
    let auth = write_auth(&dir, json!({"opencode-go": {"type": "api", "key": "super-secret"}}));
    assert_eq!(resolve_api_key(Some(&auth)).as_deref(), Some("super-secret"));
}

#[test]
fn resolve_key_falls_back_to_env() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("OPENCODE_API_KEY");
    let dir = tempdir().unwrap();
    let auth = dir.path().join("missing.json");
    std::env::set_var("OPENCODE_API_KEY", "env-key");
    let result = resolve_api_key(Some(&auth));
    std::env::remove_var("OPENCODE_API_KEY");
    assert_eq!(result.as_deref(), Some("env-key"));
}

#[test]
fn resolve_key_none_when_missing() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("OPENCODE_API_KEY");
    let dir = tempdir().unwrap();
    let auth = dir.path().join("missing.json");
    std::env::remove_var("OPENCODE_API_KEY");
    assert_eq!(resolve_api_key(Some(&auth)), None);
}

#[test]
fn resolve_key_ignores_wrong_type() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("OPENCODE_API_KEY");
    let dir = tempdir().unwrap();
    let auth = write_auth(&dir, json!({"opencode-go": {"type": "oauth", "key": "x"}}));
    std::env::remove_var("OPENCODE_API_KEY");
    assert_eq!(resolve_api_key(Some(&auth)), None);
}

#[test]
fn paths_resolve_with_dirs() {
    let _ = auth_json_path();
    let _ = default_cache_path();
}
