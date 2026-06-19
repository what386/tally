use anyhow::Result;
#[cfg(test)]
use serde::de::DeserializeOwned;
#[cfg(test)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};
use toml;

use crate::models::AppConfig;

pub struct ConfigStorage {
    config: AppConfig,
    config_file: PathBuf,
}

impl ConfigStorage {
    pub fn new(config_file: &Path) -> Result<Self> {
        let mut storage = Self {
            config: AppConfig::default(),
            config_file: config_file.to_path_buf(),
        };

        storage.load_config()?;
        Ok(storage)
    }

    /// Loads configuration from config.toml, or creates default if it doesn't exist.
    pub fn load_config(&mut self) -> Result<()> {
        if !self.config_file.exists() {
            return self.save_config();
        }

        let toml_str = fs::read_to_string(&self.config_file)
            .map_err(|e| io::Error::other(format!("Failed to load config: {}", e)))?;

        self.config = toml::from_str(&toml_str).unwrap_or_default();
        Ok(())
    }

    /// Saves the current configuration to config.toml.
    pub fn save_config(&self) -> Result<()> {
        let toml = toml::to_string_pretty(&self.config)
            .map_err(|e| io::Error::other(format!("Failed to serialize config: {}", e)))?;

        if let Some(parent) = self.config_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                io::Error::other(format!("Failed to create config directory: {}", e))
            })?;
        }

        fs::write(&self.config_file, toml)
            .map_err(|e| io::Error::other(format!("Failed to save config: {}", e)))?;

        Ok(())
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// Sets a configuration value at the given key path (e.g., "github.api_token").
    #[cfg(test)]
    pub fn try_set_value(&mut self, key_path: &str, value: &str) -> Result<(), String> {
        if key_path.trim().is_empty() {
            return Err("Key path cannot be empty".into());
        }

        let mut root = toml::Value::try_from(&self.config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        let keys: Vec<&str> = key_path.split('.').collect();
        let (path, final_key) = keys.split_at(keys.len() - 1);

        let mut current = root.as_table_mut().ok_or("Config root is not a table")?;

        for key in path {
            current = current
                .get_mut(*key)
                .and_then(toml::Value::as_table_mut)
                .ok_or_else(|| format!("Key path not found: {}", key_path))?;
        }

        let parsed_value = self.convert_value(value)?;
        current.insert(final_key[0].to_string(), parsed_value);

        self.config = root
            .try_into()
            .map_err(|e| format!("Failed to update config: {}", e))?;

        self.save_config()
            .map_err(|e| format!("Failed to save config: {}", e))
    }

    /// Gets a configuration value at the given key path.
    #[cfg(test)]
    pub fn try_get_value<T>(&self, key_path: &str) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let value = self.get_value(key_path)?;
        value
            .clone()
            .try_into()
            .map_err(|e| format!("Failed to deserialize '{}': {}", key_path, e))
    }

    #[cfg(test)]
    fn get_value(&self, key_path: &str) -> Result<toml::Value, String> {
        let root = toml::Value::try_from(&self.config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        let mut current = &root;
        for key in key_path.split('.') {
            current = current
                .get(key)
                .ok_or_else(|| format!("Key path not found: {}", key_path))?;
        }

        Ok(current.clone())
    }

    /// Gets all configuration keys and values as flattened dot-notation paths.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn get_flattened_config(&self) -> HashMap<String, String> {
        let root =
            toml::Value::try_from(&self.config).unwrap_or(toml::Value::Table(Default::default()));
        Self::flatten_value(&root, "", 10, 0)
    }

    #[cfg(test)]
    fn flatten_value(
        value: &toml::Value,
        prefix: &str,
        max_depth: usize,
        current_depth: usize,
    ) -> HashMap<String, String> {
        let mut result = HashMap::new();

        if current_depth >= max_depth {
            return result;
        }

        match value {
            toml::Value::String(s) => {
                result.insert(prefix.to_string(), s.clone());
            }
            toml::Value::Integer(i) => {
                result.insert(prefix.to_string(), i.to_string());
            }
            toml::Value::Float(f) => {
                result.insert(prefix.to_string(), f.to_string());
            }
            toml::Value::Boolean(b) => {
                result.insert(prefix.to_string(), b.to_string());
            }
            toml::Value::Table(table) => {
                for (key, val) in table {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    result.extend(Self::flatten_value(
                        val,
                        &new_prefix,
                        max_depth,
                        current_depth + 1,
                    ));
                }
            }
            _ => {}
        }

        result
    }

    #[cfg(test)]
    fn convert_value(&self, value: &str) -> Result<toml::Value, String> {
        // Try TOML literal first
        if let Ok(parsed) = value.parse::<toml::Value>() {
            return Ok(parsed);
        }

        // Fallback to string
        Ok(toml::Value::String(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(storage.get_config().scan.git_log_limit, 50);
        assert_eq!(storage.get_config().scan.todo_markers, vec!["TODO:"]);
        assert_eq!(storage.get_config().scan.done_markers, vec!["DONE:"]);
        assert_eq!(storage.get_config().matching.task_min_score, 50.0);

        cleanup(&path);
    }

    #[test]
    fn new_creates_missing_config_parent_directory() {
        let path = std::env::temp_dir()
            .join(format!(
                "tally-config-tests-missing-parent-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
            .join("nested")
            .join("config.toml");

        let storage = ConfigStorage::new(&path).unwrap();

        assert!(path.exists());
        assert_eq!(storage.get_config().git.done_prefix, "done:");

        cleanup(&path);
    }

    #[test]
    fn load_config_defaults_new_sections_for_existing_config_files() {
        let path = temp_config_path("partial-existing");
        fs::write(
            &path,
            r#"
[preferences]
auto_commit_todo = true

[git]
done_prefix = "DONE:"
"#,
        )
        .unwrap();

        let storage = ConfigStorage::new(&path).unwrap();

        assert!(storage.get_config().preferences.auto_commit_todo);
        assert_eq!(storage.get_config().git.done_prefix, "DONE:");
        assert_eq!(storage.get_config().scan.git_log_limit, 50);
        assert!(!storage.get_config().auto_commit.done);
        assert_eq!(storage.get_config().matching.released_min_score, 50.0);

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
        storage.try_set_value("auto_commit.done", "true").unwrap();
        storage.try_set_value("scan.git_log_limit", "100").unwrap();

        let auto_commit: bool = storage
            .try_get_value("preferences.auto_commit_todo")
            .unwrap();
        let done_prefix: String = storage.try_get_value("git.done_prefix").unwrap();
        let done_auto_commit: bool = storage.try_get_value("auto_commit.done").unwrap();
        let git_log_limit: usize = storage.try_get_value("scan.git_log_limit").unwrap();

        assert!(auto_commit);
        assert_eq!(done_prefix, "DONE:");
        assert!(done_auto_commit);
        assert_eq!(git_log_limit, 100);

        let reloaded = ConfigStorage::new(&path).unwrap();
        assert!(reloaded.get_config().preferences.auto_commit_todo);
        assert_eq!(reloaded.get_config().git.done_prefix, "DONE:");
        assert!(reloaded.get_config().auto_commit.done);
        assert_eq!(reloaded.get_config().scan.git_log_limit, 100);

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
}
