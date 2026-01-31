use anyhow::{Result, anyhow};
use std::env;
use std::path::PathBuf;

pub fn find_project_root() -> Result<PathBuf> {
    let mut current = env::current_dir()?;

    loop {
        let tally_dir = current.join(".tally");
        if tally_dir.exists() && tally_dir.is_dir() {
            return Ok(current);
        }

        // Try to move to parent directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                return Err(anyhow!(
                    "No .tally/ directory found. use 'tally init' to create a new one"
                ));
            }
        }
    }
}

pub struct ProjectPaths {
    pub todo_file: PathBuf,
    pub history_file: PathBuf,
    pub config_file: PathBuf,
    pub hooks_dir: PathBuf,
    pub tally_dir: PathBuf,
    pub root: PathBuf,
}

impl ProjectPaths {
    /// Get paths for the current project
    pub fn get_paths() -> Result<Self> {
        let root = find_project_root()?;
        let tally_dir = root.join(".tally");

        Ok(Self {
            todo_file: root.join("TODO.md"),
            history_file: tally_dir.join("history.json"),
            config_file: tally_dir.join("config.toml"),
            hooks_dir: tally_dir.join("hooks"),
            tally_dir,
            root,
        })
    }

    /// Initialize a new project
    pub fn init_here() -> Result<Self> {
        let root = env::current_dir()?;
        let tally_dir = root.join(".tally");
        let hooks_dir = tally_dir.join("hooks");

        if tally_dir.exists() {
            return Err(anyhow!(
                "Project already initialized at {}",
                tally_dir.display()
            ));
        }

        std::fs::create_dir_all(&tally_dir)?;
        std::fs::create_dir_all(&hooks_dir)?;

        Ok(Self {
            todo_file: root.join("TODO.md"),
            history_file: tally_dir.join("history.json"),
            config_file: tally_dir.join("config.toml"),
            hooks_dir: tally_dir.join("hooks"),
            tally_dir,
            root,
        })
    }
}
