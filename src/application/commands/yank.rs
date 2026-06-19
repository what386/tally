use crate::models::tasks::Task;
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
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    if dry_run {
        if let Some((version, change)) = changelog.remove_change(
            &description,
            None,
            tags.as_deref(),
            config.matching.released_min_score,
        ) {
            println!(
                "Would yank from {} into TODO: {}",
                version, change.description
            );
        } else {
            println!("No matching released task found.");
        }
        return Ok(());
    }

    if let Some((version, change)) = changelog.remove_change(
        &description,
        None,
        tags.as_deref(),
        config.matching.released_min_score,
    ) {
        let task = Task {
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
        };

        storage.add_task(task)?;
        changelog.save()?;

        println!(
            "Yanked from {} into TODO (semver cleared): {}",
            version, change.description
        );

        if auto || config.auto_commit_yank() {
            git::commit_tally_files("update TODO/CHANGELOG: yank released task")?;
        }

        Ok(())
    } else {
        anyhow::bail!("No matching released task found.")
    }
}
