use crate::services::storage::config_storage::ConfigStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_config_set(key: String, value: String) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ConfigStorage::new(&paths.config_file)?;
    let normalized_key = normalize_key(&key);

    storage
        .try_set_value(&normalized_key, &value)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("✓ Set {} = {}", normalized_key, value);

    Ok(())
}

pub fn cmd_config_get(key: String) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ConfigStorage::new(&paths.config_file)?;
    let normalized_key = normalize_key(&key);

    let value: String = storage
        .try_get_value(&normalized_key)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("{}", value);

    Ok(())
}

pub fn cmd_config_list() -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ConfigStorage::new(&paths.config_file)?;

    let config = storage.get_flattened_config();

    println!("Configuration:");
    for (key, value) in config {
        println!("  {}: {}", key, value);
    }

    Ok(())
}

fn normalize_key(key: &str) -> String {
    match key {
        "editor" => "preferences.editor".to_string(),
        _ => key.to_string(),
    }
}
