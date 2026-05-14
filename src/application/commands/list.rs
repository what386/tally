use crate::models::common::Priority;
use crate::models::tasks::Task;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_list(
    tags: Option<Vec<String>>,
    priority: Option<Priority>,
    done: bool,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;

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
