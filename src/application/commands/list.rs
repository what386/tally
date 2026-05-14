use crate::models::common::Priority;
use crate::models::tasks::Task;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use serde::Serialize;

pub fn cmd_list(
    tags: Option<Vec<String>>,
    priority: Option<Priority>,
    done: bool,
    released: Option<String>,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    if let Some(released_filter) = released {
        let released_tag = (released_filter != "__all__").then_some(released_filter);
        return cmd_list_released(
            &paths.changelog_file,
            storage.project_name(),
            tags,
            priority,
            released_tag,
            json,
        );
    }

    let tasks = filter_tasks(storage.tasks(), tags.as_deref(), priority.as_ref(), done);

    if json {
        let task_list: Vec<_> = tasks.iter().map(|(_, task)| task).collect();
        println!("{}", serde_json::to_string_pretty(&task_list)?);
    } else {
        if tasks.is_empty() {
            println!("No tasks found.");
            return Ok(());
        }

        for (i, task) in tasks {
            let checkbox = if task.completed { "x" } else { " " };
            let priority_str = match task.priority {
                Priority::High => " (high)",
                Priority::Medium => "",
                Priority::Low => " (low)",
            };
            let tags_str = if task.tags.is_empty() {
                String::new()
            } else {
                format!(
                    " {}",
                    task.tags
                        .iter()
                        .map(|t| format!("#{}", t))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            };

            println!(
                "{}. [{}] {}{}{}",
                i + 1,
                checkbox,
                task.description,
                priority_str,
                tags_str
            );

            if task.completed {
                if let Some(ref commit) = task.completed_at_commit {
                    println!("      @commit {}", commit);
                }
                if let Some(ref version) = task.completed_at_version {
                    println!("      @version {}", version);
                }
            }
        }
    }

    Ok(())
}

#[derive(Serialize)]
struct ReleasedEntry {
    version: String,
    description: String,
    priority: Priority,
    tags: Vec<String>,
    commit: Option<String>,
}

fn cmd_list_released(
    changelog_file: &std::path::Path,
    project_name: &str,
    tags: Option<Vec<String>>,
    priority: Option<Priority>,
    released_tag: Option<String>,
    json: bool,
) -> Result<()> {
    let changelog = ChangelogStorage::new(changelog_file, project_name)?;
    let mut entries: Vec<ReleasedEntry> = Vec::new();

    for release in changelog.log().releases.iter().rev() {
        for group in release.changes_by_priority.values() {
            for change in group {
                if let Some(filter_tags) = tags.as_ref()
                    && !filter_tags.iter().any(|tag| change.tags.contains(tag))
                {
                    continue;
                }
                if let Some(tag) = released_tag.as_ref()
                    && !change.tags.contains(tag)
                {
                    continue;
                }

                if let Some(filter_priority) = priority.as_ref()
                    && &change.priority != filter_priority
                {
                    continue;
                }

                entries.push(ReleasedEntry {
                    version: release.version.to_string(),
                    description: change.description.clone(),
                    priority: change.priority,
                    tags: change.tags.clone(),
                    commit: change.commit.clone(),
                });
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    if entries.is_empty() {
        println!("No released tasks found.");
        return Ok(());
    }

    for (i, entry) in entries.iter().enumerate() {
        let priority_str = match entry.priority {
            Priority::High => " (high)",
            Priority::Medium => "",
            Priority::Low => " (low)",
        };
        let tags_str = if entry.tags.is_empty() {
            String::new()
        } else {
            format!(
                " {}",
                entry.tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        println!(
            "{}. {}{}{} @version {}",
            i + 1,
            entry.description,
            priority_str,
            tags_str,
            entry.version
        );
        if let Some(commit) = &entry.commit {
            println!("      @commit {}", commit);
        }
    }

    Ok(())
}

fn filter_tasks<'a>(
    tasks: &'a [Task],
    tags: Option<&[String]>,
    priority: Option<&Priority>,
    done: bool,
) -> Vec<(usize, &'a Task)> {
    let mut tasks: Vec<_> = tasks.iter().enumerate().collect();

    if let Some(filter_tags) = tags {
        tasks.retain(|(_, task)| filter_tags.iter().any(|tag| task.tags.contains(tag)));
    }

    if let Some(filter_priority) = priority {
        tasks.retain(|(_, task)| &task.priority == filter_priority);
    }

    if done {
        tasks.retain(|(_, task)| task.completed);
    }

    tasks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        common::{Priority, Version},
        tasks::Task,
    };
    use chrono::{TimeZone, Utc};

    fn task(
        description: &str,
        priority: Priority,
        tags: &[&str],
        completed: bool,
        completed_version: Option<Version>,
    ) -> Task {
        Task {
            description: description.to_string(),
            priority,
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            completed,
            created_at_time: Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap(),
            created_at_version: None,
            created_at_commit: None,
            completed_at_time: completed
                .then(|| Utc.with_ymd_and_hms(2026, 4, 2, 12, 0, 0).unwrap()),
            completed_at_version: completed_version,
            completed_at_commit: None,
        }
    }

    #[test]
    fn filter_tasks_done_only_returns_completed_tasks() {
        let tasks = vec![
            task("unfinished", Priority::Medium, &["feature"], false, None),
            task(
                "finished",
                Priority::High,
                &["feature"],
                true,
                Some(Version::new(0, 6, 0, false)),
            ),
        ];

        let filtered = filter_tasks(&tasks, None, None, true);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.description, "finished");
    }

    #[test]
    fn filter_tasks_combines_filters_as_intersection() {
        let tasks = vec![
            task(
                "matching task",
                Priority::High,
                &["feature", "ux"],
                true,
                Some(Version::new(0, 6, 0, false)),
            ),
            task(
                "wrong priority",
                Priority::Medium,
                &["feature", "ux"],
                true,
                Some(Version::new(0, 6, 0, false)),
            ),
            task(
                "wrong tag",
                Priority::High,
                &["backend"],
                true,
                Some(Version::new(0, 6, 0, false)),
            ),
        ];
        let tags = vec!["feature".to_string(), "ux".to_string()];

        let filtered = filter_tasks(&tasks, Some(&tags), Some(&Priority::High), true);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.description, "matching task");
    }
}
