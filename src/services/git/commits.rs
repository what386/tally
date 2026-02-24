use crate::utils::project_paths::ProjectPaths;
use anyhow::{Result, anyhow};
use std::process::Command;

/// Auto-commits TODO.md and .tally/history.json to git
pub fn commit_tally_files(message: &str) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;

    let output = Command::new("git")
        .args([
            "commit",
            "-m",
            message,
            "--",
            "TODO.md",
            ".tally/history.json",
        ])
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
