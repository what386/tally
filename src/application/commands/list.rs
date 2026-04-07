use crate::models::common::{Priority, Version};
use crate::models::tasks::Task;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_list(
    tags: Option<Vec<String>>,
    priority: Option<Priority>,
    done: bool,
    semver: Option<String>,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let semver = semver.as_deref().map(Version::parse).transpose()?;

    let tasks = filter_tasks(
        storage.tasks(),
        tags.as_deref(),
        priority.as_ref(),
        done,
        semver.as_ref(),
    );

    if json {
        // Output as JSON
        let task_list: Vec<_> = tasks.iter().map(|(_, task)| task).collect();
        println!("{}", serde_json::to_string_pretty(&task_list)?);
    } else {
        // Human-readable output
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
    semver: Option<&Version>,
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

    if let Some(filter_semver) = semver {
        tasks.retain(|(_, task)| {
            task.completed && task.completed_at_version.as_ref() == Some(filter_semver)
        });
    }

    tasks
}

#[cfg(test)]
#[path = "../../../tests/application/commands/list_tests.rs"]
mod tests;
