use anyhow::Result;
use crate::utils::project_paths::ProjectPaths;
use crate::services::storage::task_storage::ListStorage;
use crate::services::storage::history_storage::HistoryStorage;
use crate::models::common::Version;

pub fn cmd_release(
    version_str: String,
    dry_run: bool,
    summary: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut history = HistoryStorage::new(&paths.history_file)?;

    let version = Version::parse(&version_str)?;

    // Find unversioned completed tasks
    let unversioned: Vec<_> = storage
        .tasks()
        .iter()
        .filter(|t| t.completed && t.completed_at_version.is_none())
        .collect();

    if unversioned.is_empty() {
        println!("No completed tasks without a version.");
        return Ok(());
    }

    if dry_run {
        println!(
            "Would assign version {} to {} task(s):",
            version,
            unversioned.len()
        );
        for task in &unversioned {
            println!("  [x] {}", task.description);
        }
        return Ok(());
    }

    // Record all completed tasks to history before versioning
    let task_refs: Vec<&crate::models::tasks::Task> = unversioned.iter().copied().collect();
    history.record_all(&task_refs)?;

    // Assign version in TODO.md
    let count = storage.assign_version_to_completed(version.clone())?;

    // Assign version in history.json
    history.assign_version(&version)?;

    println!("✓ Assigned version {} to {} task(s)", version, count);

    if summary {
        println!();
        println!("Tasks in {}:", version);
        for entry in history.entries_for_version(&version) {
            println!("  • {}", entry.change.description);
        }
    }

    Ok(())
}
