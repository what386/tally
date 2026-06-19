use crate::models::common::Priority;
use crate::models::tasks::Task;
use crate::services::git;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_add(
    description: String,
    priority: Option<Priority>,
    tags: Option<Vec<String>>,
    dry_run: bool,
    auto: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths().or_else(|_| ProjectPaths::for_current_dir())?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let input = parse_add_input(description, priority, tags)?;
    let task = Task::new(
        input.description.clone(),
        input.priority,
        input.tags.clone(),
    );

    if dry_run {
        println!("Would add task:");
        print_task(&task);
        return Ok(());
    }

    storage.add_task(task)?;

    if auto || config.auto_commit_add() {
        git::commit_tally_files("update TODO: add task")?;
    }

    println!("✓ Added task:");
    print_task_simple(&input.description, &input.priority, &input.tags);

    Ok(())
}

struct AddInput {
    description: String,
    priority: Priority,
    tags: Vec<String>,
}

fn parse_add_input(
    description: String,
    priority_override: Option<Priority>,
    tags_override: Option<Vec<String>>,
) -> Result<AddInput> {
    let mut description_parts = Vec::new();
    let mut parsed_priority = None;
    let mut parsed_tags = Vec::new();

    for part in description.split_whitespace() {
        if let Some(tag) = part.strip_prefix('#') {
            if !tag.is_empty() {
                parsed_tags.push(tag.to_string());
            }
            continue;
        }

        match parse_priority_marker(part) {
            Some(priority) => parsed_priority = Some(priority),
            None => description_parts.push(part),
        }
    }

    let description = description_parts.join(" ");

    if description.is_empty() {
        anyhow::bail!("Task has no description");
    }

    Ok(AddInput {
        description,
        priority: priority_override
            .or(parsed_priority)
            .unwrap_or(Priority::Medium),
        tags: tags_override.unwrap_or(parsed_tags),
    })
}

fn parse_priority_marker(value: &str) -> Option<Priority> {
    match value.to_ascii_lowercase().as_str() {
        "(high)" => Some(Priority::High),
        "(medium)" => Some(Priority::Medium),
        "(low)" => Some(Priority::Low),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_priority_and_tags() {
        let input = parse_add_input(
            "Implement a new backend (high) #backend #improvement".to_string(),
            None,
            None,
        )
        .unwrap();

        assert_eq!(input.description, "Implement a new backend");
        assert_eq!(input.priority, Priority::High);
        assert_eq!(input.tags, vec!["backend", "improvement"]);
    }

    #[test]
    fn explicit_flags_override_inline_metadata() {
        let input = parse_add_input(
            "Implement a new backend (low) #backend #improvement".to_string(),
            Some(Priority::High),
            Some(vec!["api".to_string()]),
        )
        .unwrap();

        assert_eq!(input.description, "Implement a new backend");
        assert_eq!(input.priority, Priority::High);
        assert_eq!(input.tags, vec!["api"]);
    }

    #[test]
    fn defaults_to_medium_without_inline_or_flag_priority() {
        let input = parse_add_input("Update docs #docs".to_string(), None, None).unwrap();

        assert_eq!(input.description, "Update docs");
        assert_eq!(input.priority, Priority::Medium);
        assert_eq!(input.tags, vec!["docs"]);
    }
}
