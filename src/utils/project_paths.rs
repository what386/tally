use anyhow::{Result, anyhow};
use std::env;
use std::path::PathBuf;

pub fn global_config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|dir| dir.join("tally"))
        .ok_or_else(|| anyhow!("Unable to determine global config directory"))
}

pub fn find_project_root() -> Result<PathBuf> {
    let mut current = env::current_dir()?;

    loop {
        if current.join("TODO.md").exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                return Err(anyhow!(
                    "No TODO.md found. run a write command like 'tally add' to initialize this project"
                ));
            }
        }
    }
}

pub struct ProjectPaths {
    pub todo_file: PathBuf,
    pub changelog_file: PathBuf,
    pub config_file: PathBuf,
    pub root: PathBuf,
}

impl ProjectPaths {
    pub fn get_paths() -> Result<Self> {
        let root = find_project_root()?;
        let config_dir = global_config_dir()?;

        Ok(Self {
            todo_file: root.join("TODO.md"),
            changelog_file: root.join("CHANGELOG.md"),
            config_file: config_dir.join("config.toml"),
            root,
        })
    }

    pub fn for_current_dir() -> Result<Self> {
        let root = env::current_dir()?;
        let config_dir = global_config_dir()?;

        Ok(Self {
            todo_file: root.join("TODO.md"),
            changelog_file: root.join("CHANGELOG.md"),
            config_file: config_dir.join("config.toml"),
            root,
        })
    }
}
