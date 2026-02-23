use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::services::storage::project_registry_storage::ProjectRegistryStorage;
use crate::services::storage::task_storage::ListStorage;

pub fn cmd_projects_list() -> Result<()> {
    let mut registry = ProjectRegistryStorage::new()?;
    registry.prune_missing()?;
    let projects = registry.projects();

    if projects.is_empty() {
        println!("No tally projects registered. Run `tally init` in a project first.");
        return Ok(());
    }

    println!("Registered projects:");
    for (idx, project) in projects.iter().enumerate() {
        println!("{}. {}", idx + 1, project.display());
    }

    Ok(())
}

pub fn cmd_projects_status() -> Result<()> {
    let mut registry = ProjectRegistryStorage::new()?;
    registry.prune_missing()?;
    let projects = registry.projects();

    if projects.is_empty() {
        println!("No tally projects registered. Run `tally init` in a project first.");
        return Ok(());
    }

    let mut total_tasks = 0usize;
    let mut total_open = 0usize;
    let mut total_done = 0usize;
    let mut loaded = 0usize;

    for project_root in projects {
        match project_status(&project_root) {
            Ok(summary) => {
                loaded += 1;
                total_tasks += summary.total_tasks;
                total_open += summary.open_tasks;
                total_done += summary.done_tasks;
                println!(
                    "- {} ({} v{}): {} total, {} open, {} done",
                    summary.path.display(),
                    summary.project_name,
                    summary.version,
                    summary.total_tasks,
                    summary.open_tasks,
                    summary.done_tasks
                );
            }
            Err(err) => {
                println!("- {}: skipped ({})", project_root.display(), err);
            }
        }
    }

    println!();
    if total_tasks > 0 {
        let completion = ((total_done as f64 / total_tasks as f64) * 100.0).round();
        println!(
            "Across {} project(s): {} total, {} open, {} done ({}% complete)",
            loaded, total_tasks, total_open, total_done, completion
        );
    } else {
        println!("Across {} project(s): no tasks found", loaded);
    }

    Ok(())
}

pub fn cmd_projects_prune() -> Result<()> {
    let mut registry = ProjectRegistryStorage::new()?;
    let removed = registry.prune_missing()?;
    println!("Pruned {} missing project(s) from registry.", removed);
    Ok(())
}

#[derive(Debug)]
struct ProjectSummary {
    path: PathBuf,
    project_name: String,
    version: String,
    total_tasks: usize,
    open_tasks: usize,
    done_tasks: usize,
}

fn project_status(project_root: &Path) -> Result<ProjectSummary> {
    let todo_file = project_root.join("TODO.md");
    let storage = ListStorage::new(&todo_file)?;
    let list = storage.list();

    let total_tasks = list.tasks.len();
    let done_tasks = list.tasks.iter().filter(|task| task.completed).count();
    let open_tasks = total_tasks.saturating_sub(done_tasks);

    Ok(ProjectSummary {
        path: project_root.to_path_buf(),
        project_name: list.project_name.clone(),
        version: list.project_version.to_string(),
        total_tasks,
        open_tasks,
        done_tasks,
    })
}
