use crate::models::common::Version;
use crate::output;
use crate::services::git;
use crate::services::storage::changelog_storage::ChangelogStorage;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::matching::{MatchCandidate, score_percent, select_unambiguous};
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub fn cmd_remove(
    description: String,
    released: Option<String>,
    tags: Option<Vec<String>>,
    dry_run: bool,
    auto: bool,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    if let Some(released_version_str) = released {
        let released_version = Version::parse(&released_version_str)?;
        return cmd_remove_released(description, released_version, tags, dry_run, auto, json);
    }

    let mut storage = ListStorage::new(&paths.todo_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let matcher = SkimMatcherV2::default();
    let tasks = storage.tasks();
    let mut candidates = Vec::new();

    for (i, task) in tasks.iter().enumerate() {
        if let Some(filter_tags) = tags.as_ref()
            && !filter_tags.iter().any(|tag| task.tags.contains(tag))
        {
            continue;
        }
        let exact = task.description.eq_ignore_ascii_case(&description);
        if let Some(score) = matcher
            .fuzzy_match(&task.description, &description)
            .or(exact.then_some(i64::MAX))
        {
            candidates.push(MatchCandidate {
                value: i,
                score,
                label: task.description.clone(),
                exact,
            });
        }
    }

    match select_unambiguous(candidates, config.matching.task_min_score, &description)? {
        Some(best_match) => {
            let index = best_match.value;
            let score = best_match.score;
            let score_pct = score_percent(score);

            let task = &tasks[index];

            if dry_run {
                if json {
                    return output::print_json(&serde_json::json!({
                        "status": "would_remove",
                        "dry_run": true,
                        "match_score": score_pct,
                        "task": task,
                    }));
                }
                println!("Would remove (match: {:.0}%):", score_pct);
                let checkbox = if task.completed { "x" } else { " " };
                println!("  [{}] {}", checkbox, task.description);
                return Ok(());
            }

            let removed = storage.remove_task(index)?;

            if auto || config.auto_commit_remove() {
                if json {
                    git::commit_tally_files_quiet("update TODO: remove task")?;
                } else {
                    git::commit_tally_files("update TODO: remove task")?;
                }
            }

            if let Some(task) = removed {
                if json {
                    output::print_json(&serde_json::json!({
                        "status": "removed",
                        "dry_run": false,
                        "match_score": score_pct,
                        "task": task,
                    }))?;
                } else {
                    println!("✓ Removed (match: {:.0}%): {}", score_pct, task.description);
                }
            }

            Ok(())
        }
        None => Err(anyhow::anyhow!(
            "No matching task found for: '{}'",
            description
        )),
    }
}

fn cmd_remove_released(
    description: String,
    released_version: Version,
    tags: Option<Vec<String>>,
    dry_run: bool,
    auto: bool,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let storage = ListStorage::new(&paths.todo_file)?;
    let mut changelog = ChangelogStorage::new(&paths.changelog_file, storage.project_name())?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    if dry_run {
        if let Some((v, change)) = changelog.remove_change(
            &description,
            Some(&released_version),
            tags.as_deref(),
            config.matching.released_min_score,
        )? {
            if json {
                output::print_json(&serde_json::json!({
                    "status": "would_remove_released",
                    "dry_run": true,
                    "version": v,
                    "change": change,
                }))?;
            } else {
                println!("Would remove from {}: {}", v, change.description);
            }
        } else {
            if json {
                output::print_json(&serde_json::json!({
                    "status": "not_found",
                    "dry_run": true,
                    "version": released_version,
                }))?;
            } else {
                println!("No matching released task found.");
            }
        }
        return Ok(());
    }

    if let Some((v, change)) = changelog.remove_change(
        &description,
        Some(&released_version),
        tags.as_deref(),
        config.matching.released_min_score,
    )? {
        changelog.save()?;
        if auto || config.auto_commit_remove() {
            if json {
                git::commit_tally_files_quiet("update CHANGELOG: remove released task")?;
            } else {
                git::commit_tally_files("update CHANGELOG: remove released task")?;
            }
        }
        if json {
            output::print_json(&serde_json::json!({
                "status": "removed_released",
                "dry_run": false,
                "version": v,
                "change": change,
            }))?;
        } else {
            println!("Removed from {}: {}", v, change.description);
        }
        Ok(())
    } else {
        anyhow::bail!("No matching released task found.")
    }
}
