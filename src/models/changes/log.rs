use crate::models::{changes::Release, common::Version, tasks::Task};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub project_name: String,
    pub releases: Vec<Release>,
    pub generated_at: DateTime<Utc>,
}

impl Log {
    /// Create a new changelog from a list of tasks grouped by version
    pub fn from_tasks(
        project_name: impl Into<String>,
        tasks_by_version: BTreeMap<Version, Vec<&Task>>,
    ) -> Self {
        let releases = tasks_by_version
            .into_iter()
            .map(|(version, tasks)| Release::from_tasks(version, tasks))
            .collect();

        Self {
            project_name: project_name.into(),
            releases,
            generated_at: Utc::now(),
        }
    }

    /// Filter changelog to only include releases between two versions
    pub fn filter_versions(&self, from: Option<&Version>, to: Option<&Version>) -> Self {
        let filtered_releases = self
            .releases
            .iter()
            .filter(|r| {
                let after_from = from.is_none_or(|f| &r.version >= f);
                let before_to = to.is_none_or(|t| &r.version <= t);
                after_from && before_to
            })
            .cloned()
            .collect();

        Self {
            project_name: self.project_name.clone(),
            releases: filtered_releases,
            generated_at: self.generated_at,
        }
    }
}
