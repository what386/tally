use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::{common::Version, tasks::Task};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub project_name: String,
    pub project_version: Version,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub tasks: Vec<Task>,
}

impl List {
    /// Get all tasks for a specific version
    pub fn tasks_for_version(&self, version: &Version) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| {
                t.completed_at_version
                    .as_ref()
                    .map_or(false, |v| v == version)
            })
            .collect()
    }

    /// Get all tasks between two versions (inclusive)
    pub fn tasks_between_versions(&self, from: &Version, to: &Version) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| {
                if let Some(v) = &t.completed_at_version {
                    v >= from && v <= to
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get all completed tasks without a version assigned
    pub fn unversioned_completed_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.completed && t.completed_at_version.is_none())
            .collect()
    }

    /// Get all tasks grouped by version
    pub fn tasks_by_version(&self) -> std::collections::BTreeMap<Version, Vec<&Task>> {
        let mut map = std::collections::BTreeMap::new();

        for task in &self.tasks {
            if let Some(version) = &task.completed_at_version {
                map.entry(version.clone())
                    .or_insert_with(Vec::new)
                    .push(task);
            }
        }

        map
    }

    /// Assign version to all unversioned completed tasks
    pub fn assign_version_to_completed(&mut self, version: Version) -> usize {
        let mut count = 0;

        for task in &mut self.tasks {
            if task.completed && task.completed_at_version.is_none() {
                task.completed_at_version = Some(version.clone());
                count += 1;
            }
        }

        if count > 0 {
            self.modified_at = Utc::now();
        }

        count
    }
}
