use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::utils::project_paths::ProjectPaths;
use crate::services::storage::task_storage::ListStorage;
use crate::services::storage::history_storage::HistoryStorage;

pub fn cmd_remove(
    description: String,
    dry_run: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut history = HistoryStorage::new(&paths.history_file)?;

    let matcher = SkimMatcherV2::default();
    let tasks = storage.tasks();
    let mut best_match: Option<(usize, i64)> = None;

    for (i, task) in tasks.iter().enumerate() {
        if let Some(score) = matcher.fuzzy_match(&task.description, &description) {
            if best_match.is_none() || score > best_match.unwrap().1 {
                best_match = Some((i, score));
            }
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
                if task.completed {
                    println!("  (completed task — will be saved to history first)");
                }
                return Ok(());
            }

            // If the task is completed, record it to history before removing
            if task.completed {
                history.record(task)?;
            }

            let removed = storage.remove_task(index)?;

            if let Some(task) = removed {
                println!("✓ Removed (match: {:.0}%): {}", score_pct, task.description);
            }

            Ok(())
        }
        None => Err(anyhow::anyhow!(
            "No matching task found for: '{}'",
            description
        )),
    }
}
