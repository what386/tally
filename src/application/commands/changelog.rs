use crate::models::changes::{Changelog, Release};
use crate::models::common::Version;
use crate::services::serializers::changelog_serializer;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use chrono::Utc;

pub fn cmd_changelog(from: Option<String>, to: Option<String>) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;

    let from_version = from.as_ref().map(|s| Version::parse(s)).transpose()?;
    let to_version = to.as_ref().map(|s| Version::parse(s)).transpose()?;

    // Group tasks by version
    let tasks_by_version = storage.list().tasks_by_version();

    if tasks_by_version.is_empty() {
        println!("No versioned tasks found.");
        return Ok(());
    }

    let mut releases = Vec::new();

    for (version, tasks) in tasks_by_version.iter().rev() {
        // Filter by version range if specified
        if let Some(ref from_v) = from_version {
            if version < from_v {
                continue;
            }
        }
        if let Some(ref to_v) = to_version {
            if version > to_v {
                continue;
            }
        }

        // Create release from tasks
        let release = Release::from_tasks(version.clone(), tasks.to_vec());
        releases.push(release);
    }

    if releases.is_empty() {
        println!("No releases found in the specified range.");
        return Ok(());
    }

    let changelog = Changelog {
        project_name: storage.project_name().to_string(),
        generated_at: Utc::now(),
        releases,
    };

    // Output as markdown
    let markdown = changelog_serializer::to_markdown(&changelog);
    println!("{}", markdown);

    Ok(())
}
