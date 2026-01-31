use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Embedded git hook scripts
pub struct GitHooks;

impl GitHooks {
    pub const PRE_COMMIT: &'static str = include_str!("../hooks/pre-commit");
    pub const POST_COMMIT: &'static str = include_str!("../hooks/post-commit");
    pub const PRE_PUSH: &'static str = include_str!("../hooks/pre-push");
    pub const PREPARE_COMMIT_MSG: &'static str = include_str!("../hooks/prepare-commit-msg");
}

/// Find the .git directory
fn find_git_dir() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Not in a git repository"));
    }

    let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(git_dir))
}

/// Install git hooks into .git/hooks/
pub fn install_hooks() -> Result<()> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");

    fs::create_dir_all(&hooks_dir)?;

    install_hook(&hooks_dir, "pre-commit", GitHooks::PRE_COMMIT)?;
    install_hook(&hooks_dir, "post-commit", GitHooks::POST_COMMIT)?;
    install_hook(&hooks_dir, "pre-push", GitHooks::PRE_PUSH)?;
    install_hook(&hooks_dir, "prepare-commit-msg", GitHooks::PREPARE_COMMIT_MSG)?;

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

/// Uninstall git hooks (restore backups if they exist)
pub fn uninstall_hooks() -> Result<()> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");

    uninstall_hook(&hooks_dir, "pre-commit")?;
    uninstall_hook(&hooks_dir, "post-commit")?;
    uninstall_hook(&hooks_dir, "pre-push")?;
    uninstall_hook(&hooks_dir, "prepare-commit-msg")?;

    Ok(())
}

fn uninstall_hook(hooks_dir: &Path, name: &str) -> Result<()> {
    let hook_path = hooks_dir.join(name);
    let backup_path = hooks_dir.join(format!("{}.tally-backup", name));

    if hook_path.exists() {
        // Check if it's a tally hook by looking for a marker
        if is_tally_hook(&hook_path)? {
            fs::remove_file(&hook_path)?;

            // Restore backup if exists
            if backup_path.exists() {
                fs::rename(&backup_path, &hook_path)?;
            }
        } else {
        }
    }

    Ok(())
}

/// Check if a hook file is managed by tally
fn is_tally_hook(path: &Path) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    Ok(content.contains("# tally-managed-hook") || content.contains("tally:"))
}

/// Check if hooks are installed
pub fn hooks_installed() -> Result<bool> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");

    let pre_commit = hooks_dir.join("pre-commit");
    let post_commit = hooks_dir.join("post-commit");

    Ok(pre_commit.exists() &&
       is_tally_hook(&pre_commit).unwrap_or(false) &&
       post_commit.exists() &&
       is_tally_hook(&post_commit).unwrap_or(false))
}

/// Update hooks (rewrite with latest version)
pub fn update_hooks() -> Result<()> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");

    // Only update if already installed
    if !hooks_installed()? {
        return Err(anyhow!("Hooks not installed. Run 'tally hooks install' first."));
    }

    // Rewrite without backing up (already tally hooks)
    fs::write(hooks_dir.join("pre-commit"), GitHooks::PRE_COMMIT)?;
    fs::write(hooks_dir.join("post-commit"), GitHooks::POST_COMMIT)?;
    fs::write(hooks_dir.join("pre-push"), GitHooks::PRE_PUSH)?;
    fs::write(hooks_dir.join("prepare-commit-msg"), GitHooks::PREPARE_COMMIT_MSG)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in &["pre-commit", "post-commit", "pre-push", "prepare-commit-msg"] {
            let path = hooks_dir.join(name);
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms)?;
        }
    }

    Ok(())
}

