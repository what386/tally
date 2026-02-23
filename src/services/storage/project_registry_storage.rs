use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::project_paths::global_registry_file;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProjectRegistry {
    projects: Vec<String>,
}

pub struct ProjectRegistryStorage {
    registry: ProjectRegistry,
    registry_file: PathBuf,
}

impl ProjectRegistryStorage {
    pub fn new() -> Result<Self> {
        let mut storage = Self {
            registry: ProjectRegistry::default(),
            registry_file: global_registry_file()?,
        };
        storage.load()?;
        Ok(storage)
    }

    fn load(&mut self) -> Result<()> {
        if !self.registry_file.exists() {
            self.save()?;
            return Ok(());
        }

        let raw = fs::read_to_string(&self.registry_file)?;
        if raw.trim().is_empty() {
            self.registry = ProjectRegistry::default();
            return Ok(());
        }

        self.registry = match serde_json::from_str::<ProjectRegistry>(&raw) {
            Ok(registry) => registry,
            Err(_) => {
                let legacy = serde_json::from_str::<Vec<String>>(&raw)
                    .map_err(|e| anyhow!("Failed to parse project registry: {}", e))?;
                ProjectRegistry { projects: legacy }
            }
        };

        self.normalize();
        Ok(())
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.registry_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(&self.registry)?;
        fs::write(&self.registry_file, contents)?;
        Ok(())
    }

    fn normalize(&mut self) {
        let mut deduped = Vec::new();
        for project in &self.registry.projects {
            if !deduped.contains(project) {
                deduped.push(project.clone());
            }
        }
        self.registry.projects = deduped;
    }

    fn normalize_path(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    pub fn add_project(&mut self, project_root: &Path) -> Result<bool> {
        let normalized = Self::normalize_path(project_root)
            .to_string_lossy()
            .to_string();

        if self.registry.projects.contains(&normalized) {
            return Ok(false);
        }

        self.registry.projects.push(normalized);
        self.save()?;
        Ok(true)
    }

    pub fn projects(&self) -> Vec<PathBuf> {
        self.registry.projects.iter().map(PathBuf::from).collect()
    }

    pub fn prune_missing(&mut self) -> Result<usize> {
        let before = self.registry.projects.len();
        self.registry.projects.retain(|path| {
            let root = PathBuf::from(path);
            root.join(".tally").is_dir() && root.join("TODO.md").is_file()
        });

        let removed = before.saturating_sub(self.registry.projects.len());
        if removed > 0 {
            self.save()?;
        }
        Ok(removed)
    }
}
