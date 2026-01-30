use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::models::{
    common::{Priority, Version},
    tasks::Task,
    changes::Change,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub date: DateTime<Utc>,
    pub changes_by_priority: BTreeMap<Priority, Vec<Change>>,
    pub changes_by_tag: BTreeMap<String, Vec<Change>>,
}

impl Release {
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
}
