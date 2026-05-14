use crate::models::common::Version;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub fn cmd_released(version: Option<String>, query: Option<String>) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;

    let version_filter = version.as_deref().map(Version::parse).transpose()?;
    let matcher = SkimMatcherV2::default();

    let mut found = 0;
    for release in changelog.filtered_releases(version_filter.as_ref(), version_filter.as_ref()) {
        let mut printed_header = false;
        for group in release.changes_by_priority.values() {
            for change in group {
                let matches = if let Some(q) = &query {
                    matcher
                        .fuzzy_match(&change.description, q)
                        .is_some_and(|score| score >= 40)
                } else {
                    true
                };

                if matches {
                    if !printed_header {
                        println!("{}", release.version);
                        printed_header = true;
                    }
                    println!("  - {}", change.description);
                    found += 1;
                }
            }
        }
    }

    if found == 0 {
        println!("No released tasks found.");
    }

    Ok(())
}
