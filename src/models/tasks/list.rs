use crate::models::{common::Version, tasks::Task};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub project_name: String,
    pub project_version: Version,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub tasks: Vec<Task>,
}

impl List {
    /// Create a new empty list
    pub fn new(project_name: impl Into<String>, project_version: Version) -> Self {
        let now = Utc::now();
        Self {
            project_name: project_name.into(),
            project_version,
            created_at: now,
            modified_at: now,
            tasks: Vec::new(),
        }
    }

    /// Add a task to the list
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.modified_at = Utc::now();
    }

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
    pub fn tasks_by_version(&self) -> BTreeMap<Version, Vec<&Task>> {
        let mut map = BTreeMap::new();

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
