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

}
