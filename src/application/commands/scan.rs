use crate::models::AppConfig;
use crate::models::tasks::Task;
use crate::output;
use crate::services::storage::config_storage::ConfigStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::services::{git, source};
use crate::utils::matching::{score_passes, score_percent};
use crate::utils::project_paths::ProjectPaths;
use crate::utils::task_input::parse_task_input;
use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::collections::HashSet;
use std::fmt::Write as _;

pub fn cmd_scan(auto: bool, dry_run: bool, git: bool, todo: bool, done: bool) -> Result<()> {
    let paths = ProjectPaths::get_paths().or_else(|_| ProjectPaths::for_current_dir())?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let has_selector = git || todo || done;
    let run_git = git || !has_selector;
    let run_todo = todo || !has_selector;
    let run_done = done || !has_selector;

    if run_git {
        run_git_scan(&paths.root, &mut storage, config, auto, dry_run)?;
    }

    if run_todo || run_done {
        run_source_scan(
            &paths.root,
            &mut storage,
            config,
            dry_run,
            run_todo,
            run_done,
        )?;
    }

    Ok(())
}

fn run_git_scan(
    root: &std::path::Path,
    storage: &mut ListStorage,
    config: &AppConfig,
    auto: bool,
    dry_run: bool,
) -> Result<()> {
    let commits =
        git::scan_recent_commits(root, &config.git.done_prefix, config.scan.git_log_limit)?;
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
                    if !score_passes(score, config.matching.task_min_score) {
                        continue;
                    }
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

fn run_source_scan(
    root: &std::path::Path,
    storage: &mut ListStorage,
    config: &AppConfig,
    dry_run: bool,
    include_todo: bool,
    include_done: bool,
) -> Result<()> {
    let markers = source::scan_project(root, &config.scan.todo_markers, &config.scan.done_markers)?;

    if markers.is_empty() {
        println!("No source TODO/DONE markers found.");
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
    let mut planned_done = Vec::new();
    let mut seen_new = HashSet::new();
    let matcher = SkimMatcherV2::default();

    for todo in markers {
        if todo.kind == source::SourceMarkerKind::Done {
            if !include_done {
                continue;
            }
            let match_text = parsed_source_marker_text(&todo.text);
            let mut best_match: Option<(usize, i64)> = None;
            for (idx, task) in storage.tasks().iter().enumerate() {
                if task.completed {
                    continue;
                }

                if let Some(score) = matcher.fuzzy_match(&task.description, &match_text)
                    && (best_match.is_none() || score > best_match.unwrap().1)
                {
                    best_match = Some((idx, score));
                }
            }

            if let Some((idx, score)) = best_match {
                let score_pct = score_percent(score);
                if score_pct >= config.matching.source_done_min_score {
                    planned_done.push((todo.location(), idx, todo.text.clone(), score_pct));
                }
            }
            continue;
        }

        if !include_todo {
            continue;
        }

        let task = task_from_source_todo(&todo)?;
        if existing.contains(&task.description) || seen_new.contains(&task.description) {
            if done_descriptions.contains(&task.description) {
                println!("{} - This seems like it's already done", todo.location());
            }
            continue;
        }

        seen_new.insert(task.description.clone());
        planned.push(task);
    }

    if planned.is_empty() {
        if planned_done.is_empty() {
            println!("No new source TODO tasks to add.");
            return Ok(());
        }
    }

    if dry_run {
        let mut output = String::new();
        if !planned.is_empty() {
            writeln!(output, "Would add {} source TODO task(s):", planned.len())?;
            for task in &planned {
                write_task_line(&mut output, task)?;
            }
        }
        if !planned_done.is_empty() {
            if !output.is_empty() {
                writeln!(output)?;
            }
            writeln!(
                output,
                "Would mark {} task(s) as done from source DONE markers:",
                planned_done.len()
            )?;
            for (location, idx, text, score) in &planned_done {
                writeln!(
                    output,
                    "  {} -> {} (match: {:.0}%)",
                    location,
                    storage.tasks()[*idx].description,
                    score
                )?;
                writeln!(output, "      DONE: {}", text)?;
            }
        }
        output::page_text(None, &output)?;
        return Ok(());
    }

    for task in &planned {
        storage.add_task(task.clone())?;
    }

    for (_, idx, _, _) in &planned_done {
        storage.complete_task(*idx, None)?;
    }

    if !planned.is_empty() {
        println!("Added {} source TODO task(s)", planned.len());
    }
    if !planned_done.is_empty() {
        println!(
            "Marked {} task(s) as done from source DONE markers",
            planned_done.len()
        );
    }
    Ok(())
}

fn parsed_source_marker_text(text: &str) -> String {
    parse_task_input(text, None, None)
        .map(|input| input.description)
        .unwrap_or_else(|_| text.to_string())
}

fn task_from_source_todo(todo: &source::SourceTodo) -> Result<Task> {
    let parsed = parse_task_input(&todo.text, None, None)?;
    let description = format!("{} - {}", todo.location(), parsed.description);
    Ok(Task::new(description, parsed.priority, parsed.tags))
}

fn write_task_line(output: &mut String, task: &Task) -> Result<()> {
    let priority = match task.priority {
        crate::models::common::Priority::High => " (high)",
        crate::models::common::Priority::Medium => "",
        crate::models::common::Priority::Low => " (low)",
    };

    let tags = if task.tags.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            task.tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" ")
        )
    };

    writeln!(output, "  [ ] {}{}{}", task.description, priority, tags)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Priority;

    #[test]
    fn source_todo_metadata_becomes_task_metadata() {
        let marker = source::SourceTodo {
            path: "src/main.rs".to_string(),
            line: 12,
            text: "Implement a new backend (high) #backend #improvement".to_string(),
            kind: source::SourceMarkerKind::Todo,
        };

        let task = task_from_source_todo(&marker).unwrap();

        assert_eq!(task.description, "src/main.rs:12 - Implement a new backend");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.tags, vec!["backend", "improvement"]);
    }

    #[test]
    fn source_done_matching_text_ignores_metadata() {
        assert_eq!(
            parsed_source_marker_text("Implement a new backend (high) #backend"),
            "Implement a new backend"
        );
    }
}
