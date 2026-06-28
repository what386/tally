use crate::models::common::Priority;
use crate::models::tasks::Task;
use crate::output;
use crate::services::git;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use crate::utils::task_input::parse_task_input;
use anyhow::Result;

pub fn cmd_add(
    description: String,
    priority: Option<Priority>,
    tags: Option<Vec<String>>,
    dry_run: bool,
    auto: bool,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths().or_else(|_| ProjectPaths::for_current_dir())?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let input = parse_task_input(description, priority, tags)?;
    let task = Task::new(
        input.description.clone(),
        input.priority,
        input.tags.clone(),
    );

    if dry_run {
        if json {
            return output::print_json(&serde_json::json!({
                "status": "would_add",
                "dry_run": true,
                "task": task,
            }));
        }
        println!("Would add task:");
        print_task(&task);
        return Ok(());
    }

    storage.add_task(task.clone())?;

    if auto || config.auto_commit_add() {
        if json {
            git::commit_tally_files_quiet("update TODO: add task")?;
        } else {
            git::commit_tally_files("update TODO: add task")?;
        }
    }

    if json {
        output::print_json(&serde_json::json!({
            "status": "added",
            "dry_run": false,
            "task": task,
        }))?;
    } else {
        println!("✓ Added task:");
        print_task_simple(&input.description, &input.priority, &input.tags);
    }

    Ok(())
}

fn print_task(task: &Task) {
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

    println!("  [ ] {}{}{}", task.description, priority_str, tags_str);
}

fn print_task_simple(description: &str, priority: &Priority, tags: &[String]) {
    let priority_str = match priority {
        Priority::High => " (high)",
        Priority::Medium => "",
        Priority::Low => " (low)",
    };

    let tags_str = if tags.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            tags.iter()
                .map(|t| format!("#{}", t))
                .collect::<Vec<_>>()
                .join(" ")
        )
    };

    println!("  [ ] {}{}{}", description, priority_str, tags_str);
}
