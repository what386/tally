use anyhow::Result;
use crate::utils::project_paths::ProjectPaths;
use crate::services::storage::task_storage::ListStorage;
use crate::models::common::Version;

pub fn cmd_release(
    version_str: String,
    dry_run: bool,
    summary: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;

    // Parse version
    let version = Version::parse(&version_str)?;

    // Count unversioned completed tasks
    let unversioned: Vec<_> = storage.tasks()
        .iter()
        .filter(|t| t.completed && t.completed_at_version.is_none())
        .collect();

    if unversioned.is_empty() {
        println!("No completed tasks without a version.");
        return Ok(());
    }

    if dry_run {
        println!("Would assign version {} to {} task(s):", version, unversioned.len());
        for task in unversioned {
            println!("  [x] {}", task.description);
        }
        return Ok(());
    }

    let count = storage.assign_version_to_completed(version.clone())?;

    println!("✓ Assigned version {} to {} task(s)", version, count);

    if summary {
        println!();
        println!("Tasks in {}:", version);
        for task in storage.tasks_for_version(&version) {
            println!("  • {}", task.description);
        }
    }

    Ok(())
}
