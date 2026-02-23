use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::env;
use std::io::ErrorKind;
use std::process::Command;

use crate::services::storage::config_storage::ConfigStorage;
use crate::utils::project_paths::ProjectPaths;

pub fn cmd_edit() -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let mut candidates = Vec::new();

    if let Some(editor) = config
        .preferences
        .editor
        .as_ref()
        .map(String::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        candidates.push(editor.to_string());
    }

    if let Ok(editor) = env::var("EDITOR") {
        let editor = editor.trim();
        if !editor.is_empty() {
            candidates.push(editor.to_string());
        }
    }

    for editor in ["nvim", "vim", "nano", "vi"] {
        candidates.push(editor.to_string());
    }

    let mut seen = HashSet::new();
    let unique_candidates: Vec<String> = candidates
        .into_iter()
        .filter(|editor| seen.insert(editor.clone()))
        .collect();

    for candidate in unique_candidates {
        let mut parts = candidate.split_whitespace();
        let Some(command) = parts.next() else {
            continue;
        };
        let args: Vec<&str> = parts.collect();

        match Command::new(command)
            .args(args)
            .arg(&paths.todo_file)
            .status()
        {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => {
                return Err(anyhow!(
                    "Editor '{}' exited with non-zero status: {}",
                    candidate,
                    status
                ));
            }
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => return Err(anyhow!("Failed to launch editor '{}': {}", candidate, err)),
        }
    }

    Err(anyhow!(
        "No editor available. Set 'preferences.editor' with `tally config set preferences.editor <editor>` or export EDITOR"
    ))
}
