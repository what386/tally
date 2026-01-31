use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use crate::models::changes::Change;
use crate::models::common::Version;
use crate::models::tasks::Task;

/// History storage persists completed tasks to history.json
/// so that changelog generation works even after prune.
pub struct HistoryStorage {
    entries: Vec<HistoryEntry>,
    history_file: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub change: Change,
    pub version: Option<Version>,
}

impl HistoryEntry {
    pub fn from_task(task: &Task) -> Self {
        Self {
            change: Change::from(task),
            version: task.completed_at_version.clone(),
        }
    }
}

impl HistoryStorage {
    pub fn new(history_file: &Path) -> Result<Self> {
        let mut storage = Self {
            entries: Vec::new(),
            history_file: history_file.to_path_buf(),
        };
        storage.load()?;
        Ok(storage)
    }

    fn load(&mut self) -> Result<()> {
        if !self.history_file.exists() {
            self.entries = Vec::new();
            return Ok(());
        }

        let content = fs::read_to_string(&self.history_file)?;
        self.entries = serde_json::from_str(&content).unwrap_or_default();
        Ok(())
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.history_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| anyhow!("Failed to serialize history: {}", e))?;
        fs::write(&self.history_file, json)
            .map_err(|e| anyhow!("Failed to write history file: {}", e))?;
        Ok(())
    }

    /// Record a completed task into history
    pub fn record(&mut self, task: &Task) -> Result<()> {
        // Don't record incomplete tasks
        if !task.completed {
            return Ok(());
        }

        // Don't duplicate: check if this exact task is already recorded
        // Match on description + completed_at_time
        let already_exists = self.entries.iter().any(|e| {
            e.change.description == task.description
                && e.change.completed_at == task.completed_at_time.unwrap_or_default()
        });

        if already_exists {
            return Ok(());
        }

        self.entries.push(HistoryEntry::from_task(task));
        self.save()
    }

    /// Record multiple tasks at once
    pub fn record_all(&mut self, tasks: &[&Task]) -> Result<()> {
        for task in tasks {
            if !task.completed {
                continue;
            }

            let already_exists = self.entries.iter().any(|e| {
                e.change.description == task.description
                    && e.change.completed_at == task.completed_at_time.unwrap_or_default()
            });

            if !already_exists {
                self.entries.push(HistoryEntry::from_task(task));
            }
        }
        self.save()
    }

    /// Assign a version to all unversioned entries
    pub fn assign_version(&mut self, version: &Version) -> Result<usize> {
        let mut count = 0;
        for entry in &mut self.entries {
            if entry.version.is_none() {
                entry.version = Some(version.clone());
                count += 1;
            }
        }
        if count > 0 {
            self.save()?;
        }
        Ok(count)
    }

    /// Get all entries for a specific version
    pub fn entries_for_version(&self, version: &Version) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.version.as_ref() == Some(version))
            .collect()
    }

    /// Get all entries between two versions (inclusive)
    pub fn entries_between_versions(
        &self,
        from: &Version,
        to: &Version,
    ) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(v) = &e.version {
                    v >= from && v <= to
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get all versioned entries grouped by version
    pub fn entries_by_version(&self) -> std::collections::BTreeMap<Version, Vec<&HistoryEntry>> {
        let mut map = std::collections::BTreeMap::new();
        for entry in &self.entries {
            if let Some(version) = &entry.version {
                map.entry(version.clone())
                    .or_insert_with(Vec::new)
                    .push(entry);
            }
        }
        map
    }

    /// Get all unversioned entries
    pub fn unversioned_entries(&self) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.version.is_none()).collect()
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
