use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{enums::Priority, version::Version};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changelog {
    pub version_from: Version,
    pub version_to: Version,
    pub date: DateTime<Utc>,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub description: String,
    pub priority: Priority,
    pub tags: Vec<String>,

    pub completed_at: DateTime<Utc>,
    pub completed_at_version: Option<Version>,
    pub completed_at_commit: Option<String>,
}
