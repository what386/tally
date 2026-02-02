use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::utils::project_paths::ProjectPaths;

/// Embedded git hook scripts
pub const POST_COMMIT: &str = include_str!("scripts/post-commit");

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

    install_hook(&hooks_dir, "post-commit", POST_COMMIT)?;

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

    uninstall_hook(&hooks_dir, "post-commit")?;

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

    fs::write(hooks_dir.join("post-commit"), POST_COMMIT)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in &[
            "post-commit",
        ] {
            let path = hooks_dir.join(name);
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms)?;
        }
    }

    Ok(())
}
