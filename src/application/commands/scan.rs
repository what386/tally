use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::process::Command;
use crate::services::storage::ignore_storage::IgnoreStorage;
use crate::services::storage::task_storage::ListStorage;
use crate::services::storage::history_storage::HistoryStorage;
use crate::utils::project_paths::ProjectPaths;

#[derive(Debug)]
struct Commit {
    hash: String,
    done_items: Vec<String>,
    date: DateTime<Utc>,
}

pub fn cmd_scan(auto: bool, dry_run: bool) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;
    let mut history = HistoryStorage::new(&paths.history_file)?;
    let ignore = IgnoreStorage::load(&paths.ignore_file);

    let output = Command::new("git")
        .args(["log", "--pretty=format:%h%x1f%ct%x1f%B%x1e", "-n", "50"])
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("failed to read git log");
    }

    let raw = String::from_utf8(output.stdout)?;
    let commits = parse_commits(&raw);
    let matcher = SkimMatcherV2::default();
    let mut matches_found = 0;
    let mut completed = Vec::new();

    for (idx, task) in storage.tasks().iter().enumerate() {
        if task.completed {
            continue;
        }

        if ignore.is_ignored(&task.description, &task.tags) {
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
                println!("  (dry-run: would mark as done)");
            } else if auto {
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
                    println!("  → Skipped");
                }
            }

            println!();
        }
    }

    if !dry_run {
        for (idx, hash) in &completed {
            storage.complete_task(*idx, None)?;
            if let Some(task) = storage.tasks_mut().get_mut(*idx) {
                task.completed_at_commit = Some(hash.clone());
            }
        }
        storage.save_list()?;

        // Record completed tasks to history after all mutations are done
        for (idx, _) in &completed {
            if let Some(task) = storage.tasks().get(*idx) {
                history.record(task)?;
            }
        }
    }

    if matches_found == 0 {
        println!("No matches found.");
    }

    Ok(())
}

fn parse_commits(input: &str) -> Vec<Commit> {
    let mut commits = Vec::new();

    for record in input.split('\x1e') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }

        let mut parts = record.splitn(3, '\x1f'); // 3 parts: hash, timestamp, body
        let hash = parts.next().unwrap().to_string();
        let timestamp_str = parts.next().unwrap_or("0");
        let body = parts.next().unwrap_or("").trim();

        let ts: i64 = timestamp_str.parse().unwrap_or(0);
        let date: DateTime<Utc> = Utc
            .timestamp_opt(ts, 0)
            .single()
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());

        let done_items = extract_done_items(body);
        if !done_items.is_empty() {
            commits.push(Commit {
                hash,
                done_items,
                date,
            });
        }
    }

    commits
}

fn extract_done_items(message: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_done = false;

    for line in message.lines() {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("done:") {
            in_done = true;
            continue;
        }

        if in_done {
            if trimmed.is_empty() {
                break;
            }
            // Stop at next section header
            if trimmed.ends_with(':') {
                break;
            }
            let cleaned = trimmed.trim_start_matches(['-', '*']).trim().to_string();
            if !cleaned.is_empty() {
                items.push(cleaned);
            }
        }
    }

    items
}
