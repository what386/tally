use crate::models::{
    changes::Change,
    common::{Priority, Version},
    tasks::Task,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub date: DateTime<Utc>,
    pub changes_by_priority: BTreeMap<Priority, Vec<Change>>,
    pub changes_by_tag: BTreeMap<String, Vec<Change>>,
}

impl Release {
    /// Build a Release from a list of tasks (used when reading directly from TODO.md)
    pub fn from_tasks(version: Version, tasks: Vec<&Task>) -> Self {
        let mut changes_by_priority = BTreeMap::new();
        let mut changes_by_tag = BTreeMap::new();

        let date = tasks
            .iter()
            .filter_map(|t| t.completed_at_time)
            .max()
            .unwrap_or_else(Utc::now);

        for task in tasks {
            let change = Change::from(task);
            changes_by_priority
                .entry(task.priority)
                .or_insert_with(Vec::new)
                .push(change.clone());

            for tag in &task.tags {
                changes_by_tag
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(change.clone());
            }
        }

        Self {
            version,
            date,
            changes_by_priority,
            changes_by_tag,
        }
    }

    /// Build a Release from a list of changes (used when reading from history.json)
    pub fn from_changes(version: Version, date: DateTime<Utc>, changes: Vec<&Change>) -> Self {
        let mut changes_by_priority = BTreeMap::new();
        let mut changes_by_tag = BTreeMap::new();

        for change in changes {
            changes_by_priority
                .entry(change.priority)
                .or_insert_with(Vec::new)
                .push(change.clone());

            for tag in &change.tags {
                changes_by_tag
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(change.clone());
            }
        }

        Self {
            version,
            date,
            changes_by_priority,
            changes_by_tag,
        }
    }
}
