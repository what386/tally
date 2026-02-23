use std::fs::File;

use crate::models::common::Version;
use crate::models::tasks::List;
use crate::services::git::hooks;
use crate::services::serializers::todo_serializer;
use crate::services::storage::project_registry_storage::ProjectRegistryStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_init() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let had_tally_dir = current_dir.join(".tally").exists();
    let had_todo = current_dir.join("TODO.md").exists();
    let had_history = current_dir.join(".tally").join("history.json").exists();

    println!("Initializing tally project...");

    // Create project structure
    let paths = ProjectPaths::init_here()?;

    // Detect project name from directory
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled")
        .to_string();

    if !had_tally_dir {
        println!("Created .tally/ directory structure");
    }

    if !had_history {
        File::create(&paths.history_file)?;
        println!("Created .tally/history.json");
    }

    if !had_todo {
        let initial_list = List::new(&project_name, Version::new(0, 1, 0, false));
        let content = todo_serializer::serialize(&initial_list);
        std::fs::write(&paths.todo_file, content)?;
        println!("Created TODO.md");
    } else {
        println!("Using existing TODO.md");
    }

    let mut registry = ProjectRegistryStorage::new()?;
    if registry.add_project(&paths.root)? {
        println!("Registered project in ~/.config/tally/projects.json");
    } else {
        println!("Project already in ~/.config/tally/projects.json");
    }

    // Install git hooks if in a git repository
    match hooks::install_hooks() {
        Ok(()) => {
            println!("Git integration enabled:");
            println!("  - Commit messages with 'done:' section will auto-complete tasks");
        }
        Err(e) => {
            println!("⚠ Git hooks not installed: {}", e);
            println!("  (This is OK if you're not using git)");
        }
    }

    println!();
    if had_tally_dir && had_todo && had_history {
        println!("Tally already initialized here. Registry and hooks are up to date.");
    } else {
        println!("Tally initialized! Try:");
        println!("  tally add \"My first task\"");
        println!("  tally list");
    }

    Ok(())
}
