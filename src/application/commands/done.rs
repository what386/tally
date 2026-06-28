use crate::models::common::Version;
use crate::output;
use crate::services::git;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::matching::{MatchCandidate, score_percent, select_unambiguous};
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub fn cmd_done(
    description: String,
    commit: Option<String>,
    version: Option<String>,
    dry_run: bool,
    auto: bool,
    json: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    // Fuzzy match the description
    let matcher = SkimMatcherV2::default();
    let tasks = storage.tasks();
    let mut candidates = Vec::new();

    for (i, task) in tasks.iter().enumerate() {
        if task.completed {
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
            let task = &tasks[index];
            let score_pct = score_percent(score);

            if dry_run {
                if json {
                    return output::print_json(&serde_json::json!({
                        "status": "would_complete",
                        "dry_run": true,
                        "match_score": score_pct,
                        "task": task,
                        "completed_commit": commit,
                        "completed_version": version,
                    }));
                }
                println!("Would mark as done (score: {:.0}%):", score_pct);
                println!("  [x] {}", task.description);
                if let Some(ref commit_hash) = commit {
                    println!("      @completed_commit {}", commit_hash);
                }
                if let Some(ref v) = version {
                    println!("      @completed_version {}", v);
                }
                return Ok(());
            }

            let version_obj = if let Some(v) = version {
                Some(Version::parse(&v)?)
            } else {
                None
            };

            let description = task.description.clone();

            // Add commit hash if provided
            if let Some(commit_hash) = commit {
                if let Some(task) = storage.tasks_mut().get_mut(index) {
                    task.completed_at_commit = Some(commit_hash);
                }
                storage.save_list()?;
            }

            storage.complete_task(index, version_obj)?;
            let completed_task = storage.tasks()[index].clone();

            if auto || config.auto_commit_done() {
                if json {
                    git::commit_tally_files_quiet("update TODO: complete task")?;
                } else {
                    git::commit_tally_files("update TODO: complete task")?;
                }
            }

            if json {
                output::print_json(&serde_json::json!({
                    "status": "completed",
                    "dry_run": false,
                    "match_score": score_pct,
                    "task": completed_task,
                }))?;
            } else {
                println!("Marked as done: {}", description);
            }

            Ok(())
        }
        None => Err(anyhow::anyhow!(
            "No matching task found for: '{}'",
            description
        )),
    }
}
