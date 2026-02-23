use crate::models::common::Version;
use crate::services::git::commits;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::history_storage::HistoryStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub fn cmd_done(
    description: String,
    commit: Option<String>,
    version: Option<String>,
    dry_run: bool,
    auto: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut history = HistoryStorage::new(&paths.history_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    // Fuzzy match the description
    let matcher = SkimMatcherV2::default();
    let tasks = storage.tasks();
    let mut best_match: Option<(usize, i64)> = None;

    for (i, task) in tasks.iter().enumerate() {
        if task.completed {
            continue;
        }

        if let Some(score) = matcher.fuzzy_match(&task.description, &description)
            && (best_match.is_none() || score > best_match.unwrap().1)
        {
            best_match = Some((i, score));
        }
    }

    match best_match {
        Some((index, score)) => {
            let task = &tasks[index];

            if dry_run {
                println!("Would mark as done (score: {:.0}):", score);
                println!("  [x] {}", task.description);
                if let Some(ref commit_hash) = commit {
                    println!("      @completed_commit {}", commit_hash);
                }
                if let Some(ref v) = version {
                    println!("      @completed_version {}", v);
                }
                return Ok(());
            }

            let version_obj = if let Some(v) = version {
                Some(Version::parse(&v)?)
            } else {
                None
            };

            // occurs early because evil borrow semantics
            println!("Marked as done: {}", task.description);

            // Add commit hash if provided
            if let Some(commit_hash) = commit {
                if let Some(task) = storage.tasks_mut().get_mut(index) {
                    task.completed_at_commit = Some(commit_hash);
                }
                storage.save_list()?;
            }

            storage.complete_task(index, version_obj)?;

            // Record to history after all mutations are done
            if let Some(task) = storage.tasks().get(index) {
                history.record(task)?;
            }

            if auto || config.preferences.auto_commit_todo {
                commits::commit_tally_files("update TODO: complete task")?;
            }

            Ok(())
        }
        None => Err(anyhow::anyhow!(
            "No matching task found for: '{}'",
            description
        )),
    }
}
