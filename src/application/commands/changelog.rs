use crate::models::changes::Log;
use crate::models::common::Version;
use crate::services::serializers::changelog_serializer;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use chrono::Utc;

pub fn cmd_changelog(from: Option<String>, to: Option<String>) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;

    let from_version = from.as_ref().map(|s| Version::parse(s)).transpose()?;
    let to_version = to.as_ref().map(|s| Version::parse(s)).transpose()?;

    let releases = changelog.filtered_releases(from_version.as_ref(), to_version.as_ref());

    if releases.is_empty() {
        println!("No releases found in the specified range.");
        return Ok(());
    }

    let filtered = Log {
        project_name: changelog.log().project_name.clone(),
        generated_at: Utc::now(),
        releases,
    };

    let markdown = changelog_serializer::to_markdown(&filtered);
    println!("{}", markdown);

    Ok(())
}
