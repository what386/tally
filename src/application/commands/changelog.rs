use crate::models::changes::{Log, Release};
use crate::models::common::Version;
use crate::services::serializers::changelog_serializer;
use crate::services::storage::history_storage::HistoryStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use chrono::Utc;

pub fn cmd_changelog(from: Option<String>, to: Option<String>) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let history = HistoryStorage::new(&paths.history_file)?;

    let from_version = from.as_ref().map(|s| Version::parse(s)).transpose()?;
    let to_version = to.as_ref().map(|s| Version::parse(s)).transpose()?;

    let entries_by_version = history.entries_by_version();

    if entries_by_version.is_empty() {
        println!("No versioned entries found in history.");
        return Ok(());
    }

    let mut releases = Vec::new();

    for (version, entries) in entries_by_version.iter().rev() {
        if let Some(ref from_v) = from_version
            && version < from_v {
                continue;
            }
        if let Some(ref to_v) = to_version
            && version > to_v {
                continue;
            }

        // Build tasks from history entries for Release::from_tasks
        // We construct temporary Task-like data via the changes
        let changes: Vec<&crate::models::changes::Change> =
            entries.iter().map(|e| &e.change).collect();

        let date = changes
            .iter()
            .map(|c| c.completed_at)
            .max()
            .unwrap_or_else(Utc::now);

        let release = Release::from_changes(version.clone(), date, changes);
        releases.push(release);
    }

    if releases.is_empty() {
        println!("No releases found in the specified range.");
        return Ok(());
    }

    let changelog = Log {
        project_name: storage.project_name().to_string(),
        generated_at: Utc::now(),
        releases,
    };

    let markdown = changelog_serializer::to_markdown(&changelog);
    println!("{}", markdown);

    Ok(())
}
