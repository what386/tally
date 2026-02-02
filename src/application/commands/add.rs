use crate::models::common::Priority;
use crate::models::tasks::Task;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_add(
    description: String,
    priority: Priority,
    tags: Option<Vec<String>>,
    dry_run: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;

    let task = Task::new(
        description.clone(),
        priority,
        tags.clone().unwrap_or_default(),
    );

    if dry_run {
        println!("Would add task:");
        print_task(&task);
        return Ok(());
    }

    let mut storage = ListStorage::new(&paths.todo_file)?;
    storage.add_task(task)?;

    println!("✓ Added task:");
    print_task_simple(&description, &priority, &tags.unwrap_or_default());

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
