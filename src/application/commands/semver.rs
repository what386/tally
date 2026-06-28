use anyhow::Result;

use crate::models::common::Version;
use crate::output;
use crate::services::git;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use std::fmt::Write as _;

pub fn cmd_semver(
    version_str: String,
    dry_run: bool,
    summary: bool,
    auto: bool,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let version = Version::parse(&version_str)?;

    let unversioned_indices: Vec<usize> = storage
        .tasks()
        .iter()
        .enumerate()
        .filter(|(_, t)| t.completed && t.completed_at_version.is_none())
        .map(|(idx, _)| idx)
        .collect();

    if unversioned_indices.is_empty() {
        if json {
            return output::print_json(&serde_json::json!({
                "status": "noop",
                "dry_run": dry_run,
                "version": version,
                "tasks": [],
            }));
        }
        println!("Nothing to do: No completed tasks without a version.");
        return Ok(());
    }

    let selected_tasks: Vec<_> = unversioned_indices
        .iter()
        .map(|idx| storage.tasks()[*idx].clone())
        .collect();

    if dry_run {
        if json {
            return output::print_json(&serde_json::json!({
                "status": "would_release",
                "dry_run": true,
                "version": version,
                "tasks": selected_tasks,
            }));
        }
        let mut output = String::new();
        writeln!(
            output,
            "Would assign version {} and move {} task(s) to CHANGELOG.md:",
            version,
            unversioned_indices.len()
        )?;
        for idx in &unversioned_indices {
            writeln!(output, "  [x] {}", storage.tasks()[*idx].description)?;
        }
        output::page_text(None, &output)?;
        return Ok(());
    }

    for idx in &unversioned_indices {
        if let Some(task) = storage.tasks_mut().get_mut(*idx) {
            task.completed_at_version = Some(version.clone());
        }
    }

    let changes = selected_tasks
        .iter()
        .map(crate::models::changes::Change::from)
        .collect();
    let inserted = changelog.merge_changes_for_version(&version, changes);

    let mut removal_indices = unversioned_indices;
    removal_indices.sort_unstable_by(|a, b| b.cmp(a));
    for idx in removal_indices {
        storage.remove_task(idx)?;
    }

    changelog.save()?;

    if auto || config.auto_commit_semver() {
        if json {
            git::commit_tally_files_quiet("update TODO/CHANGELOG: set semver")?;
        } else {
            git::commit_tally_files("update TODO/CHANGELOG: set semver")?;
        }
    }

    if json {
        output::print_json(&serde_json::json!({
            "status": "released",
            "dry_run": false,
            "version": version,
            "inserted": inserted,
            "tasks": selected_tasks,
        }))?;
    } else {
        println!(
            "Moved {} task(s) into CHANGELOG.md under version {}",
            inserted, version
        );
    }

    if summary && !json {
        let mut output = String::new();
        writeln!(output)?;
        writeln!(output, "Tasks moved into {}:", version)?;
        for task in selected_tasks {
            writeln!(output, "  • {}", task.description)?;
        }
        output::page_text(None, &output)?;
    }

    Ok(())
}
