use crate::models::{common::Version, tasks::Task};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
