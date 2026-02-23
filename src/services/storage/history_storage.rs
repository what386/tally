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

    fn is_same_entry(a: &HistoryEntry, b: &HistoryEntry) -> bool {
        match (&a.change.commit, &b.change.commit) {
            (Some(left), Some(right)) => left == right,
            _ => a.change.description == b.change.description && a.version == b.version,
        }
    }

    fn insert_if_new(&mut self, task: &Task) {
        let candidate = HistoryEntry::from_task(task);
        let already_exists = self
            .entries
            .iter()
            .any(|entry| Self::is_same_entry(entry, &candidate));

        if !already_exists {
            self.entries.push(candidate);
        }
    }

    /// Record a completed task into history
    pub fn record(&mut self, task: &Task) -> Result<()> {
        // Don't record incomplete tasks
        if !task.completed {
            return Ok(());
        }

        self.insert_if_new(task);
        self.save()
    }

    /// Record multiple tasks at once
    pub fn record_all(&mut self, tasks: &[&Task]) -> Result<()> {
        for task in tasks {
            if !task.completed {
                continue;
            }
            self.insert_if_new(task);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Priority;
    use chrono::{TimeZone, Utc};

    fn make_task(
        description: &str,
        commit: Option<&str>,
        version: Option<Version>,
        completed_at: chrono::DateTime<Utc>,
    ) -> Task {
        Task {
            description: description.to_string(),
            priority: Priority::High,
            tags: vec![],
            completed: true,
            created_at_time: completed_at,
            created_at_version: None,
            created_at_commit: None,
            completed_at_time: Some(completed_at),
            completed_at_version: version,
            completed_at_commit: commit.map(str::to_string),
        }
    }

    fn make_entry(
        description: &str,
        commit: Option<&str>,
        version: Option<Version>,
        completed_at: chrono::DateTime<Utc>,
    ) -> HistoryEntry {
        HistoryEntry {
            change: Change {
                description: description.to_string(),
                priority: Priority::High,
                tags: vec![],
                commit: commit.map(str::to_string),
                completed_at,
            },
            version,
        }
    }

    #[test]
    fn record_all_dedupes_when_commit_matches() {
        let version = Version::new(0, 5, 0, false);
        let first_time = Utc.with_ymd_and_hms(2026, 2, 2, 19, 19, 32).unwrap();
        let second_time = Utc.with_ymd_and_hms(2026, 2, 2, 19, 19, 0).unwrap();

        let mut storage = HistoryStorage {
            entries: vec![make_entry(
                "github doing!",
                Some("9ba2c4a"),
                Some(version.clone()),
                first_time,
            )],
            history_file: std::env::temp_dir().join("tally-history-record-all-commit.json"),
        };

        let task = make_task(
            "github doing!",
            Some("9ba2c4a"),
            Some(version),
            second_time,
        );
        storage.record_all(&[&task]).unwrap();

        assert_eq!(storage.entries.len(), 1);
    }

    #[test]
    fn record_all_dedupes_when_description_and_version_match() {
        let first_time = Utc.with_ymd_and_hms(2026, 2, 2, 2, 15, 14).unwrap();
        let second_time = Utc.with_ymd_and_hms(2026, 2, 2, 2, 15, 0).unwrap();

        let mut storage = HistoryStorage {
            entries: vec![make_entry("config support", None, None, first_time)],
            history_file: std::env::temp_dir().join("tally-history-record-all-desc-version.json"),
        };

        let task = make_task("config support", None, None, second_time);
        storage.record_all(&[&task]).unwrap();

        assert_eq!(storage.entries.len(), 1);
    }

    #[test]
    fn record_all_keeps_distinct_versions() {
        let old_version = Version::new(0, 4, 0, false);
        let new_version = Version::new(0, 5, 0, false);
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 2, 2, 15, 0).unwrap();

        let mut storage = HistoryStorage {
            entries: vec![make_entry(
                "release notes update",
                None,
                Some(old_version),
                timestamp,
            )],
            history_file: std::env::temp_dir().join("tally-history-record-all-versions.json"),
        };

        let task = make_task(
            "release notes update",
            None,
            Some(new_version),
            timestamp,
        );
        storage.record_all(&[&task]).unwrap();

        assert_eq!(storage.entries.len(), 2);
    }
}
