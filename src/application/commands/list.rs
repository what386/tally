use anyhow::Result;
use crate::utils::project_paths::ProjectPaths;
use crate::services::storage::task_storage::ListStorage;
use crate::models::common::Priority;

pub fn cmd_list(
    tags: Option<Vec<String>>,
    priority: Option<Priority>,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;

    let mut tasks: Vec<_> = storage.tasks().iter().enumerate().collect();

    // Filter by tags
    if let Some(ref filter_tags) = tags {
        tasks.retain(|(_, task)| {
            filter_tags.iter().any(|tag| task.tags.contains(tag))
        });
    }

    // Filter by priority
    if let Some(ref filter_priority) = priority {
        tasks.retain(|(_, task)| &task.priority == filter_priority);
    }

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
                format!(" {}", task.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "))
            };

            println!("{}. [{}] {}{}{}",
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
