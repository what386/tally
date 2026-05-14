use crate::models::common::Version;
use crate::services::git::commits;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;

pub fn cmd_unrelease(
    description: String,
    version: Option<String>,
    dry_run: bool,
    auto: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let mut changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let parsed_version = version.as_deref().map(Version::parse).transpose()?;

    if dry_run {
        if let Some((v, change)) = changelog.remove_change(&description, parsed_version.as_ref()) {
            println!("Would remove from {}: {}", v, change.description);
        } else {
            println!("No matching released task found.");
        }
        return Ok(());
    }

    if let Some((v, change)) = changelog.remove_change(&description, parsed_version.as_ref()) {
        changelog.save()?;
        println!("Removed from {}: {}", v, change.description);
        if auto || config.preferences.auto_commit_todo {
            commits::commit_tally_files("update CHANGELOG: unrelease task")?;
        }
        Ok(())
    } else {
        anyhow::bail!("No matching released task found.")
    }
}
