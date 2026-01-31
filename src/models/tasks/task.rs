use crate::models::common::{Priority, Version};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub description: String,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub completed: bool,

    // Creation metadata
    pub created_at_time: DateTime<Utc>,
    pub created_at_version: Option<Version>,
    pub created_at_commit: Option<String>,

    // Completion metadata
    pub completed_at_time: Option<DateTime<Utc>>,
    pub completed_at_version: Option<Version>,
    pub completed_at_commit: Option<String>,
}

impl Task {
    /// Create a new incomplete task
    pub fn new(description: impl Into<String>, priority: Priority, tags: Vec<String>) -> Self {
        Self {
            description: description.into(),
            priority,
            tags,
            completed: false,
            created_at_time: Utc::now(),
            created_at_version: None,
            created_at_commit: None,
            completed_at_time: None,
            completed_at_version: None,
            completed_at_commit: None,
        }
    }

    /// Mark task as complete
    pub fn mark_complete(&mut self, commit: Option<String>, version: Option<Version>) {
        self.completed = true;
        self.completed_at_time = Some(Utc::now());
        self.completed_at_commit = commit;
        self.completed_at_version = version;
    }

    /// Check if task matches any of the given tags
    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter().any(|t| self.tags.contains(t))
    }

    /// Check if task has all given tags
    pub fn has_all_tags(&self, tags: &[String]) -> bool {
        tags.iter().all(|t| self.tags.contains(t))
    }

    /// Check if task is older than given duration (in days)
    pub fn is_older_than_days(&self, days: u32) -> bool {
        if let Some(completed_time) = self.completed_at_time {
            let age = Utc::now().signed_duration_since(completed_time);
            age.num_days() > days as i64
        } else {
            false
        }
    }
}
