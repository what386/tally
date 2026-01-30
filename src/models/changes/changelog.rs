use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::models::{common::{Priority, Version}, tasks::Task};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changelog {
    pub project_name: String,
    pub releases: Vec<Release>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    pub date: DateTime<Utc>,
    pub tasks_by_priority: BTreeMap<Priority, Vec<ChangelogEntry>>,
    pub tasks_by_tag: BTreeMap<String, Vec<ChangelogEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub description: String,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub commit: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl From<&Task> for ChangelogEntry {
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

impl Changelog {
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
        let filtered_releases = self.releases
            .iter()
            .filter(|r| {
                let after_from = from.map_or(true, |f| &r.version >= f);
                let before_to = to.map_or(true, |t| &r.version <= t);
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

    /// Convert to markdown format
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("# Changelog — {}\n\n", self.project_name));
        output.push_str(&format!(
            "*Generated on {}*\n\n",
            self.generated_at.format("%Y-%m-%d")
        ));

        for release in &self.releases {
            output.push_str(&release.to_markdown());
            output.push('\n');
        }

        output
    }

    /// Convert to JSON format
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl Release {
    fn from_tasks(version: Version, tasks: Vec<&Task>) -> Self {
        let mut tasks_by_priority = BTreeMap::new();
        let mut tasks_by_tag = BTreeMap::new();

        // Get the latest completion date as the release date
        let date = tasks
            .iter()
            .filter_map(|t| t.completed_at_time)
            .max()
            .unwrap_or_else(Utc::now);

        for task in tasks {
            let entry = ChangelogEntry::from(*task);

            // Group by priority
            tasks_by_priority
                .entry(task.priority)
                .or_insert_with(Vec::new)
                .push(entry.clone());

            // Group by tags
            for tag in &task.tags {
                tasks_by_tag
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(entry.clone());
            }
        }

        Self {
            version,
            date,
            tasks_by_priority,
            tasks_by_tag,
        }
    }

    fn to_markdown(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("## {} — {}\n\n", self.version, self.date.format("%Y-%m-%d")));

        // Group by priority (High -> Medium -> Low)
        for priority in [Priority::High, Priority::Medium, Priority::Low] {
            if let Some(entries) = self.tasks_by_priority.get(&priority) {
                if !entries.is_empty() {
                    let priority_name = match priority {
                        Priority::High => "High Priority",
                        Priority::Medium => "Changes",
                        Priority::Low => "Minor Changes",
                    };

                    output.push_str(&format!("### {}\n\n", priority_name));

                    for entry in entries {
                        let tags = if entry.tags.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", entry.tags.join(", "))
                        };

                        let commit = entry.commit
                            .as_ref()
                            .map(|c| format!(" ({})", &c[..7.min(c.len())]))
                            .unwrap_or_default();

                        output.push_str(&format!("- {}{}{}\n", entry.description, tags, commit));
                    }

                    output.push('\n');
                }
            }
        }

        output
    }
}
