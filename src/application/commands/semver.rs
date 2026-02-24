use std::process::Command;

use anyhow::{Result, anyhow};

use crate::models::common::Version;
use crate::services::git::commits;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::history_storage::HistoryStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;

pub fn cmd_semver(version_str: String, dry_run: bool, summary: bool, auto: bool) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut history = HistoryStorage::new(&paths.history_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let version = Version::parse(&version_str)?;

    storage.set_project_version(version.clone())?;

    println!("Set project version to '{}'", &version);

    // Find unversioned completed tasks
    let unversioned: Vec<_> = storage
        .tasks()
        .iter()
        .filter(|t| t.completed && t.completed_at_version.is_none())
        .collect();

    if unversioned.is_empty() {
        println!("Nothing to do: No completed tasks without a version.");
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
    let task_refs: Vec<&crate::models::tasks::Task> = unversioned.to_vec();
    history.record_all(&task_refs)?;

    // Assign version in TODO.md
    let count = storage.assign_version_to_completed(version.clone())?;

    // Assign version in history.json
    history.assign_version(&version)?;

    println!("Assigned version {} to {} task(s)", version, count);

    if summary {
        println!();
        println!("Tasks in {}:", version);
        for entry in history.entries_for_version(&version) {
            println!("  • {}", entry.change.description);
        }
    }

    if auto || config.preferences.auto_commit_todo {
        commits::commit_tally_files("update TODO: set semver")?;
    }

    Ok(())
}

pub fn cmd_tag(
    version_str: String,
    message: Option<String>,
    dry_run: bool,
    summary: bool,
    auto: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;

    let tag_name = if version_str.starts_with('v') {
        version_str.clone()
    } else {
        format!("v{}", version_str)
    };

    // Run release
    cmd_semver(version_str.clone(), dry_run, summary, auto)?;

    let msg = message.unwrap_or_else(|| format!("Release {}", tag_name));

    if dry_run {
        println!();
        println!("Would commit TODO.md and .tally/history.json");
        println!("Would create git tag: {} — {}", tag_name, msg);
        return Ok(());
    }

    // Create annotated tag
    let output = Command::new("git")
        .args(["tag", "-a", &tag_name, "-m", &msg])
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to create git tag: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("✓ Created git tag: {}", tag_name);

    Ok(())
}
