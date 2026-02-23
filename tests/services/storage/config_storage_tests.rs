use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_config_path(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("tally-config-tests-{test_name}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join("config.toml")
}

fn cleanup(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

#[test]
fn new_creates_default_config_file() {
    let path = temp_config_path("default-create");

    let storage = ConfigStorage::new(&path).unwrap();

    assert!(path.exists());
    assert_eq!(storage.get_config().git.done_prefix, "done:");
    assert!(!storage.get_config().preferences.auto_commit_todo);

    cleanup(&path);
}

#[test]
fn set_get_and_reload_round_trip() {
    let path = temp_config_path("round-trip");

    let mut storage = ConfigStorage::new(&path).unwrap();
    storage
        .try_set_value("preferences.auto_commit_todo", "true")
        .unwrap();
    storage
        .try_set_value("git.done_prefix", "\"DONE:\"")
        .unwrap();

    let auto_commit: bool = storage
        .try_get_value("preferences.auto_commit_todo")
        .unwrap();
    let done_prefix: String = storage.try_get_value("git.done_prefix").unwrap();

    assert!(auto_commit);
    assert_eq!(done_prefix, "DONE:");

    let reloaded = ConfigStorage::new(&path).unwrap();
    assert!(reloaded.get_config().preferences.auto_commit_todo);
    assert_eq!(reloaded.get_config().git.done_prefix, "DONE:");

    cleanup(&path);
}

#[test]
fn rejects_invalid_or_empty_key_paths() {
    let path = temp_config_path("invalid-paths");

    let mut storage = ConfigStorage::new(&path).unwrap();

    let empty_err = storage.try_set_value("", "true").unwrap_err();
    assert!(empty_err.contains("Key path cannot be empty"));

    let path_err = storage
        .try_set_value("preferences.missing.leaf", "true")
        .unwrap_err();
    assert!(path_err.contains("Key path not found"));

    cleanup(&path);
}
