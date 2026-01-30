use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::{common::Priority, tasks::Task};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub description: String,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub commit: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl From<&Task> for Change {
    fn from(task: &Task) -> Self {
        Self {
            description: task.description.clone(),
            priority: task.priority,
            tags: task.tags.clone(),
            commit: task.completed_at_commit.clone(),
            completed_at: task.completed_at_time.unwrap_or_else(Utc::now),
        }
    }
}
