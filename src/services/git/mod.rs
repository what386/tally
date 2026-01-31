use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::utils::project_paths::ProjectPaths;

/// Embedded git hook scripts
pub const PRE_COMMIT: &str = include_str!("hooks/pre-commit");
pub const POST_COMMIT: &str = include_str!("hooks/post-commit");
pub const PRE_PUSH: &str = include_str!("hooks/pre-push");
pub const PREPARE_COMMIT_MSG: &str = include_str!("hooks/prepare-commit-msg");

fn set_git_hooks_path(repo_path: &Path, hooks_path: &Path) -> std::io::Result<()> {
    let status = Command::new("git")
        .arg("config")
        .arg("core.hooksPath")
        .arg(hooks_path)
        .current_dir(repo_path)
        .status()?; // run in the repo directory

    if !status.success() {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to set core.hooksPath",
        ))
    } else {
        Ok(())
    }
}

/// Install git hooks into .git/hooks/
pub fn install_hooks() -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let hooks_dir = paths.hooks_dir;

    fs::create_dir_all(&hooks_dir)?;

    install_hook(&hooks_dir, "pre-commit", PRE_COMMIT)?;
    install_hook(&hooks_dir, "post-commit", POST_COMMIT)?;
    install_hook(&hooks_dir, "pre-push", PRE_PUSH)?;
    install_hook(&hooks_dir, "prepare-commit-msg", PREPARE_COMMIT_MSG)?;

    set_git_hooks_path(&paths.root.join(".git"), &hooks_dir)?;

    Ok(())
}

/// Install a single hook, backing up if exists
fn install_hook(hooks_dir: &Path, name: &str, content: &str) -> Result<()> {
    let hook_path = hooks_dir.join(name);

    // Backup existing hook if present
    if hook_path.exists() {
        let backup_path = hooks_dir.join(format!("{}.tally-backup", name));
        fs::copy(&hook_path, backup_path)?;
    }

    // Write new hook
    fs::write(&hook_path, content)?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

/// Uninstall git hooks
pub fn uninstall_hooks() -> Result<()> {
    let hooks_dir = ProjectPaths::get_paths()?.hooks_dir;

    uninstall_hook(&hooks_dir, "pre-commit")?;
    uninstall_hook(&hooks_dir, "post-commit")?;
    uninstall_hook(&hooks_dir, "pre-push")?;
    uninstall_hook(&hooks_dir, "prepare-commit-msg")?;

    Ok(())
}

fn uninstall_hook(hooks_dir: &Path, name: &str) -> Result<()> {
    let hook_path = hooks_dir.join(name);

    fs::remove_file(&hook_path)?;

    Ok(())
}


/// Update hooks (rewrite with latest version)
pub fn update_hooks() -> Result<()> {
    let hooks_dir = ProjectPaths::get_paths()?.hooks_dir;

    fs::write(hooks_dir.join("pre-commit"), PRE_COMMIT)?;
    fs::write(hooks_dir.join("post-commit"), POST_COMMIT)?;
    fs::write(hooks_dir.join("pre-push"), PRE_PUSH)?;
    fs::write(hooks_dir.join("prepare-commit-msg"), PREPARE_COMMIT_MSG)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in &[
            "pre-commit",
            "post-commit",
            "pre-push",
            "prepare-commit-msg",
        ] {
            let path = hooks_dir.join(name);
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms)?;
        }
    }

    Ok(())
}
