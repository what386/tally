use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use std::process::Command;

pub fn cmd_edit() -> Result<()> {
    let paths = ProjectPaths::get_paths()?;

    // Try to find an editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Try common editors
            for cmd in &["vim", "nvim", "nano", "emacs", "code"] {
                if Command::new("which").arg(cmd).output().is_ok() {
                    return cmd.to_string();
                }
            }
            "vim".to_string()
        });

    let status = Command::new(editor).arg(&paths.todo_file).status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    Ok(())
}
