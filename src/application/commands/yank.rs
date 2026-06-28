use crate::models::changes::Change;
use crate::models::tasks::Task;
use crate::output;
use crate::services::git;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use chrono::Utc;

pub fn cmd_yank(
    description: String,
    tags: Option<Vec<String>>,
    dry_run: bool,
    auto: bool,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    if dry_run {
        let changes = changelog.remove_changes(
            &description,
            None,
            tags.as_deref(),
            config.matching.released_min_score,
        )?;
        if json {
            return output::print_json(&serde_json::json!({
                "status": if changes.is_empty() { "not_found" } else { "would_yank" },
                "dry_run": true,
                "changes": changes,
            }));
        }
        if changes.is_empty() {
            println!("No matching released task found.");
        } else if changes.len() == 1 {
            let (version, change) = &changes[0];
            println!(
                "Would yank from {} into TODO: {}",
                version, change.description
            );
        } else {
            println!(
                "Would yank {} task(s) from {} into TODO:",
                changes.len(),
                changes[0].0
            );
            for (_, change) in &changes {
                println!("- {}", change.description);
            }
        }
        return Ok(());
    }

    let changes = changelog.remove_changes(
        &description,
        None,
        tags.as_deref(),
        config.matching.released_min_score,
    )?;

    if !changes.is_empty() {
        let tasks: Vec<Task> = changes
            .iter()
            .map(|(_, change)| task_from_change(change))
            .collect();
        storage.add_tasks(tasks.clone())?;
        changelog.save()?;

        if auto || config.auto_commit_yank() {
            if json {
                git::commit_tally_files_quiet("update TODO/CHANGELOG: yank released task")?;
            } else {
                git::commit_tally_files("update TODO/CHANGELOG: yank released task")?;
            }
        }

        if json {
            output::print_json(&serde_json::json!({
                "status": "yanked",
                "dry_run": false,
                "changes": changes,
                "tasks": tasks,
            }))?;
        } else if changes.len() == 1 {
            let (version, change) = &changes[0];
            println!(
                "Yanked from {} into TODO (semver cleared): {}",
                version, change.description
            );
        } else {
            println!(
                "Yanked {} task(s) from {} into TODO (semver cleared)",
                changes.len(),
                changes[0].0
            );
        }

        Ok(())
    } else {
        anyhow::bail!("No matching released task found.")
    }
}

fn task_from_change(change: &Change) -> Task {
    Task {
        description: change.description.clone(),
        priority: change.priority,
        tags: change.tags.clone(),
        completed: true,
        created_at_time: Utc::now(),
        created_at_version: None,
        created_at_commit: None,
        completed_at_time: Some(change.completed_at),
        completed_at_version: None,
        completed_at_commit: change.commit.clone(),
    }
}
