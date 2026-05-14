use crate::services::git;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub fn cmd_remove(
    description: String,
    released: Option<String>,
    dry_run: bool,
    auto: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    if let Some(released_filter) = released {
        let released_tag = (released_filter != "__all__").then_some(released_filter);
        return cmd_remove_released(description, released_tag, dry_run, auto);
    }

    let mut storage = ListStorage::new(&paths.todo_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let matcher = SkimMatcherV2::default();
    let tasks = storage.tasks();
    let mut best_match: Option<(usize, i64)> = None;

    for (i, task) in tasks.iter().enumerate() {
        if let Some(score) = matcher.fuzzy_match(&task.description, &description)
            && (best_match.is_none() || score > best_match.unwrap().1)
        {
            best_match = Some((i, score));
        }
    }

    match best_match {
        Some((index, score)) => {
            let score_pct = (score as f64).min(100.0);

            if score_pct < 50.0 {
                return Err(anyhow::anyhow!(
                    "Best match too low ({:.0}%): '{}'",
                    score_pct,
                    tasks[index].description
                ));
            }

            let task = &tasks[index];

            if dry_run {
                println!("Would remove (match: {:.0}%):", score_pct);
                let checkbox = if task.completed { "x" } else { " " };
                println!("  [{}] {}", checkbox, task.description);
                return Ok(());
            }

            let removed = storage.remove_task(index)?;

            if let Some(task) = removed {
                println!("✓ Removed (match: {:.0}%): {}", score_pct, task.description);
            }

            if auto || config.preferences.auto_commit_todo {
                git::commit_tally_files("update TODO: remove task")?;
            }

            Ok(())
        }
        None => Err(anyhow::anyhow!(
            "No matching task found for: '{}'",
            description
        )),
    }
}

fn cmd_remove_released(
    description: String,
    released_tag: Option<String>,
    dry_run: bool,
    auto: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let mut changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    if dry_run {
        if let Some((v, change)) =
            changelog.remove_change(&description, None, released_tag.as_deref())
        {
            println!("Would remove from {}: {}", v, change.description);
        } else {
            println!("No matching released task found.");
        }
        return Ok(());
    }

    if let Some((v, change)) = changelog.remove_change(&description, None, released_tag.as_deref())
    {
        changelog.save()?;
        println!("Removed from {}: {}", v, change.description);
        if auto || config.preferences.auto_commit_todo {
            git::commit_tally_files("update CHANGELOG: remove released task")?;
        }
        Ok(())
    } else {
        anyhow::bail!("No matching released task found.")
    }
}
