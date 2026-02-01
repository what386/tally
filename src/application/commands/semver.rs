use std::process::Command;

use anyhow::{Result, anyhow};

use crate::utils::project_paths::ProjectPaths;
use crate::services::storage::task_storage::ListStorage;
use crate::services::storage::history_storage::HistoryStorage;
use crate::models::common::Version;

pub fn cmd_semver(
    version_str: String,
    dry_run: bool,
    summary: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut history = HistoryStorage::new(&paths.history_file)?;

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
    let task_refs: Vec<&crate::models::tasks::Task> = unversioned.iter().copied().collect();
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

    Ok(())
}

pub fn cmd_tag(
    version_str: String,
    message: Option<String>,
    dry_run: bool,
    summary: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;

    let tag_name = if version_str.starts_with('v') {
        version_str.clone()
    } else {
        format!("v{}", version_str)
    };

    // Check for uncommitted changes outside of TODO.md and history.json
    // so we don't silently commit in a dirty working tree
    let tally_files = ["TODO.md", ".tally/history.json"];

    let dirty: Vec<String> = {
        let output = Command::new("git")
            .args(["diff", "--name-only"])
            .current_dir(&paths.root)
            .output()?;
        String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|f| !tally_files.contains(&f.as_str()))
            .collect()
    };

    if !dirty.is_empty() {
        return Err(anyhow!(
            "Working tree has uncommitted changes:\n{}\n\
             Commit or stash them before tagging.",
            dirty.join("\n")
        ));
    }

    let staged: Vec<String> = {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(&paths.root)
            .output()?;
        String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|f| !tally_files.contains(&f.as_str()))
            .collect()
    };

    if !staged.is_empty() {
        return Err(anyhow!(
            "Working tree has staged changes:\n{}\n\
             Commit or stash them before tagging.",
            staged.join("\n")
        ));
    }

    // Run release
    cmd_semver(version_str.clone(), dry_run, summary)?;

    let msg = message.unwrap_or_else(|| format!("Release {}", tag_name));

    if dry_run {
        println!();
        println!("Would commit TODO.md and .tally/history.json");
        println!("Would create git tag: {} — {}", tag_name, msg);
        return Ok(());
    }

    // Stage TODO.md and history.json
    let output = Command::new("git")
        .args(["add", "TODO.md", ".tally/history.json"])
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to stage files: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Commit
    let commit_msg = format!("Release {}", tag_name);
    let output = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("Committed TODO.md and .tally/history.json");

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
