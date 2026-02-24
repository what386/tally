use crate::models::changes::Release;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub project_name: String,
    pub releases: Vec<Release>,
    pub generated_at: DateTime<Utc>,
}
