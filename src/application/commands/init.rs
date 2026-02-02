use std::fs::File;

use crate::models::common::Version;
use crate::models::tasks::List;
use crate::services::git::hooks;
use crate::services::serializers::todo_serializer;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_init() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Check if already initialized
    if current_dir.join(".tally").exists() {
        println!("Tally project already initialized in this directory");
        return Ok(());
    }

    println!("Initializing tally project...");

    // Create project structure
    let paths = ProjectPaths::init_here()?;

    // Detect project name from directory
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled")
        .to_string();

    // create empty history
    File::create(paths.history_file)?;

    // Create initial TODO.md
    let initial_list = List::new(&project_name, Version::new(0, 1, 0, false));
    let content = todo_serializer::serialize(&initial_list);
    std::fs::write(&paths.todo_file, content)?;

    println!("Created .tally/ directory structure");
    println!("Created .tally/history.json");
    println!("Created TODO.md");

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
    println!("Tally initialized! Try:");
    println!("  tally add \"My first task\"");
    println!("  tally list");

    Ok(())
}
