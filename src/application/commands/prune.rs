use anyhow::Result;
use chrono::Utc;
use crate::services::git::commits;
use crate::services::storage::config_storage::ConfigStorage;
use crate::utils::project_paths::ProjectPaths;
use crate::services::storage::task_storage::ListStorage;
use crate::services::storage::history_storage::HistoryStorage;

pub fn cmd_prune(
    days: Option<u32>,
    hours: Option<u32>,
    dry_run: bool,
    auto: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut history = HistoryStorage::new(&paths.history_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    // Build the cutoff duration from flags
    // If both are provided, they add together
    let total_hours = match (days, hours) {
        (Some(d), Some(h)) => (d as u64) * 24 + h as u64,
        (Some(d), None) => (d as u64) * 24,
        (None, Some(h)) => h as u64,
        (None, None) => {
            // Default: 30 days
            30 * 24
        }
    };

    let cutoff = Utc::now()
        - chrono::Duration::hours(total_hours as i64);

    // Find completed tasks older than cutoff
    let to_prune: Vec<(usize, &crate::models::tasks::Task)> = storage
        .tasks()
        .iter()
        .enumerate()
        .filter(|(_, task)| {
            task.completed
                && task
                    .completed_at_time
                    .map_or(false, |t| t < cutoff)
        })
        .collect();

    if to_prune.is_empty() {
        println!("No completed tasks older than {} to prune.", format_duration(total_hours));
        return Ok(());
    }

    if dry_run {
        println!(
            "Would prune {} completed task(s) older than {}:\n",
            to_prune.len(),
            format_duration(total_hours)
        );
        for (_, task) in &to_prune {
            let completed_str = task
                .completed_at_time
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            println!("  [x] {} (completed: {})", task.description, completed_str);
        }
        return Ok(());
    }

    // Record all prunable tasks to history before removing
    let task_refs: Vec<&crate::models::tasks::Task> =
        to_prune.iter().map(|(_, task)| *task).collect();
    history.record_all(&task_refs)?;

    // Remove in reverse index order so indices stay valid
    let mut indices: Vec<usize> = to_prune.iter().map(|(i, _)| *i).collect();
    indices.sort_unstable_by(|a, b| b.cmp(a));

    for index in &indices {
        storage.remove_task(*index)?;
    }

    if auto || config.preferences.auto_commit_todo {
        commits::commit_tally_files("update TODO: prune tasks")?;
    }

    println!(
        "✓ Pruned {} completed task(s) older than {}",
        indices.len(),
        format_duration(total_hours)
    );
    println!("  All pruned tasks saved to history.json");

    Ok(())
}

fn format_duration(total_hours: u64) -> String {
    let days = total_hours / 24;
    let hours = total_hours % 24;

    match (days, hours) {
        (0, h) => format!("{} hour(s)", h),
        (d, 0) => format!("{} day(s)", d),
        (d, h) => format!("{} day(s) {} hour(s)", d, h),
    }
}
