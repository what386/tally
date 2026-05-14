use crate::utils::project_paths::ProjectPaths;
use anyhow::{Result, anyhow};
use std::process::Command;

pub fn commit_tally_files(message: &str) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;

    let mut files = vec!["TODO.md"];
    if paths.changelog_file.exists() {
        files.push("CHANGELOG.md");
    }

    let mut args = vec!["commit", "-m", message, "--"];
    args.extend(files.iter().copied());

    let output = Command::new("git")
        .args(args)
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("Committed {}", files.join(" and "));

    Ok(())
}
