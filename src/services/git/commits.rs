use std::process::Command;
use anyhow::{Result, anyhow};
use crate::utils::project_paths::ProjectPaths;

/// Auto-commits TODO.md and .tally/history.json to git
pub fn commit_tally_files(message: &str) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;

    let output = Command::new("git")
        .args(["commit", "-m", message, "--", "TODO.md", ".tally/history.json"])
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("Committed TODO.md and .tally/history.json");

    Ok(())
}

/// Checks if there are uncommitted or staged changes outside of tally files
pub fn check_for_non_tally_changes() -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let tally_files = ["TODO.md", ".tally/history.json"];

    // Check unstaged changes
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
             Commit or stash them first.",
            dirty.join("\n")
        ));
    }

    // Check staged changes
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
             Commit or stash them first.",
            staged.join("\n")
        ));
    }

    Ok(())
}
