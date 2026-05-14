use crate::models::common::Priority;
use crate::models::tasks::Task;
use crate::services::{git, source};
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::collections::HashSet;

pub fn cmd_scan(auto: bool, dry_run: bool, git: bool, source_only: bool) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;

    let run_git = git || (!git && !source_only);
    let run_source = source_only || (!git && !source_only);

    if run_git {
        run_git_scan(&paths.root, &paths.config_file, &mut storage, auto, dry_run)?;
    }

    if run_source {
        run_source_scan(&paths.root, &mut storage, dry_run)?;
    }

    Ok(())
}

fn run_git_scan(
    root: &std::path::Path,
    config_file: &std::path::Path,
    storage: &mut ListStorage,
    auto: bool,
    dry_run: bool,
) -> Result<()> {
    let config_storage = ConfigStorage::new(config_file)?;
    let config = config_storage.get_config();

    let commits = git::scan_recent_commits(root, &config.git.done_prefix)?;
    let matcher = SkimMatcherV2::default();
    let mut matches_found = 0;
    let mut completed = Vec::new();

    for (idx, task) in storage.tasks().iter().enumerate() {
        if task.completed {
            continue;
        }

        let mut best_match: Option<(String, i64, String)> = None;

        for commit in &commits {
            if commit.date < task.created_at_time {
                continue;
            }

            for done in &commit.done_items {
                if let Some(score) = matcher.fuzzy_match(&task.description, done) {
                    let is_better = best_match
                        .as_ref()
                        .map(|(_, best, _)| score > *best)
                        .unwrap_or(true);
                    if is_better {
                        best_match = Some((commit.hash.clone(), score, done.clone()));
                    }
                }
            }
        }

        if let Some((hash, score, done_line)) = best_match {
            matches_found += 1;

            println!("Match found (score: {}):", score);
            println!("  Task: {}", task.description);
            println!("  Done: {}", done_line);
            println!("  Commit: {}", hash);

            if dry_run {
                println!("  (dry-run: would mark as done)\n");
                continue;
            }

            if auto || config.preferences.auto_complete_tasks {
                completed.push((idx, hash));
            } else {
                use std::io::{self, Write};
                print!("  Mark as done? [y/N]: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if input.trim().eq_ignore_ascii_case("y") {
                    completed.push((idx, hash));
                } else {
                    println!("  -> Skipped");
                }
            }

            println!();
        }
    }

    for (idx, hash) in &completed {
        storage.complete_task(*idx, None)?;
        if let Some(task) = storage.tasks_mut().get_mut(*idx) {
            task.completed_at_commit = Some(hash.clone());
        }
    }
    storage.save_list()?;

    if matches_found == 0 {
        println!("No git commit matches found.");
    }

    Ok(())
}

fn run_source_scan(root: &std::path::Path, storage: &mut ListStorage, dry_run: bool) -> Result<()> {
    let todos = source::scan_project(root)?;

    if todos.is_empty() {
        println!("No source TODO markers found.");
        return Ok(());
    }

    let mut existing = HashSet::new();
    let mut done_descriptions = HashSet::new();
    for task in storage.tasks() {
        existing.insert(task.description.clone());
        if task.completed {
            done_descriptions.insert(task.description.clone());
        }
    }

    let mut planned = Vec::new();
    let mut seen_new = HashSet::new();

    for todo in todos {
        let desc = todo.description();
        if existing.contains(&desc) || seen_new.contains(&desc) {
            if done_descriptions.contains(&desc) {
                println!(
                    "{} - This seems like it's already done",
                    todo.location()
                );
            }
            continue;
        }

        seen_new.insert(desc.clone());
        planned.push((todo.location(), desc));
    }

    if planned.is_empty() {
        println!("No new source TODO tasks to add.");
        return Ok(());
    }

    if dry_run {
        println!("Would add {} source TODO task(s):", planned.len());
        for (_, desc) in &planned {
            println!("  [ ] {}", desc);
        }
        return Ok(());
    }

    for (_, desc) in &planned {
        storage.add_task(Task::new(desc.clone(), Priority::Medium, vec![]))?;
    }

    println!("Added {} source TODO task(s)", planned.len());
    Ok(())
}

