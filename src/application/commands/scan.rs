use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::process::Command;
use crate::utils::project_paths::ProjectPaths;
use crate::services::storage::task_storage::ListStorage;

pub fn cmd_scan(
    auto: bool,
    threshold: f64,
    dry_run: bool,
) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let mut storage = ListStorage::new(&paths.todo_file)?;

    // Get recent commit messages
    let output = Command::new("git")
        .args(&["log", "--pretty=%h|%s", "-n", "50"])
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to read git log"));
    }

    let commits = String::from_utf8(output.stdout)?;
    let matcher = SkimMatcherV2::default();

    let mut matches_found = 0;
    let mut completed_indices = Vec::new();

    for (idx, task) in storage.tasks().iter().enumerate() {
        if task.completed {
            continue;
        }

        // Find best matching commit
        let mut best_commit: Option<(String, String, f64)> = None;

        for line in commits.lines() {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() != 2 {
                continue;
            }

            let (hash, message) = (parts[0], parts[1]);

            if let Some(score) = matcher.fuzzy_match(message, &task.description) {
                let score_pct = (score as f64).min(100.0);

                if score_pct >= threshold * 100.0 {
                    if best_commit.is_none() || score_pct > best_commit.as_ref().unwrap().2 {
                        best_commit = Some((hash.to_string(), message.to_string(), score_pct));
                    }
                }
            }
        }

        if let Some((hash, message, score)) = best_commit {
            matches_found += 1;

            println!("Match found ({:.0}%):", score);
            println!("  Task:   {}", task.description);
            println!("  Commit: {} - {}", hash, message);

            if dry_run {
                println!("  (dry-run: would mark as done)");
            } else if auto {
                // Auto-complete
                println!("  → Marking as done");
                completed_indices.push((idx, hash.clone()));
            } else {
                // Prompt user
                print!("  Mark as done? [y/N]: ");
                use std::io::{self, Write};
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().eq_ignore_ascii_case("y") {
                    completed_indices.push((idx, hash.clone()));
                } else {
                    println!("  → Skipped");
                }
            }

            println!();
        }
    }

    // Complete the tasks
    if !dry_run {
        for (idx, hash) in completed_indices {
            storage.complete_task(idx, None)?;
            if let Some(task) = storage.tasks_mut().get_mut(idx) {
                task.completed_at_commit = Some(hash);
            }
        }
        storage.save_list()?;
    }

    if matches_found == 0 {
        println!("No matches found above {:.0}% threshold.", threshold * 100.0);
    }

    Ok(())
}
