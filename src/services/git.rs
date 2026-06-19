use crate::models::app_config::TrackCreatedFiles;
use crate::output;
use crate::services::storage::config_storage::ConfigStorage;
use crate::utils::project_paths::ProjectPaths;
use anyhow::{Result, anyhow, bail};
use chrono::{DateTime, TimeZone, Utc};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub done_items: Vec<String>,
    pub date: DateTime<Utc>,
}

pub fn commit_tally_files(message: &str) -> Result<()> {
    let paths = ProjectPaths::get_paths()?;
    let config_storage = ConfigStorage::new(&paths.config_file)?;
    let config = config_storage.get_config();

    let mut files = vec!["TODO.md"];
    if paths.changelog_file.exists() {
        files.push("CHANGELOG.md");
    }

    track_untracked_tally_files(&paths.root, &files, config.git.track_created_files)?;

    let mut args = vec!["commit", "-m", message, "--"];
    args.extend(files.iter().copied());

    let output = Command::new("git")
        .args(args)
        .current_dir(&paths.root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("Committed {}", files.join(" and "));

    Ok(())
}

fn track_untracked_tally_files(
    root: &Path,
    files: &[&str],
    policy: TrackCreatedFiles,
) -> Result<()> {
    let untracked = untracked_files(root, files)?;
    if untracked.is_empty() {
        return Ok(());
    }

    let file_list = untracked.join(", ");
    match policy {
        TrackCreatedFiles::Always => {}
        TrackCreatedFiles::Never => {
            bail!("Cannot auto-commit untracked tally file(s): {file_list}");
        }
        TrackCreatedFiles::Prompt => {
            let prompt = format!("Track newly created tally file(s) with git: {file_list}?");
            if !output::confirm(prompt, true)? {
                bail!("Cannot auto-commit untracked tally file(s): {file_list}");
            }
        }
    }

    let mut args = vec!["add", "--"];
    args.extend(untracked.iter().map(String::as_str));

    let output = Command::new("git").args(args).current_dir(root).output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "Failed to track tally file(s): {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn untracked_files(root: &Path, files: &[&str]) -> Result<Vec<String>> {
    let mut untracked = Vec::new();

    for file in files {
        let output = Command::new("git")
            .args(["ls-files", "--error-unmatch", "--", file])
            .current_dir(root)
            .output()?;

        if !output.status.success() {
            untracked.push((*file).to_string());
        }
    }

    Ok(untracked)
}

pub fn scan_recent_commits(
    root: &Path,
    done_marker: &str,
    git_log_limit: usize,
) -> Result<Vec<CommitEntry>> {
    let limit = git_log_limit.to_string();
    let output = Command::new("git")
        .args(["log", "--pretty=format:%h%x1f%ct%x1f%B%x1e", "-n", &limit])
        .current_dir(root)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("failed to read git log");
    }

    let raw = String::from_utf8(output.stdout)?;
    Ok(parse_commits(&raw, done_marker))
}

fn parse_commits(input: &str, done_marker: &str) -> Vec<CommitEntry> {
    let mut commits = Vec::new();

    for record in input.split('\x1e') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }

        let mut parts = record.splitn(3, '\x1f');
        let hash = parts.next().unwrap().to_string();
        let timestamp_str = parts.next().unwrap_or("0");
        let body = parts.next().unwrap_or("").trim();

        let ts: i64 = timestamp_str.parse().unwrap_or(0);
        let date: DateTime<Utc> = Utc
            .timestamp_opt(ts, 0)
            .single()
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());

        let done_items = extract_done_items(body, done_marker);
        if !done_items.is_empty() {
            commits.push(CommitEntry {
                hash,
                done_items,
                date,
            });
        }
    }

    commits
}

fn extract_done_items(message: &str, done_marker: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_done = false;

    for line in message.lines() {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case(done_marker) {
            in_done = true;
            continue;
        }

        if in_done {
            if trimmed.is_empty() {
                break;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_done_items_parses_list_and_stops_on_blank_line() {
        let message =
            "feat: improve parser\n\nDone:\n- first item\n* second item\n\nNotes:\n- not included";

        let items = extract_done_items(message, "done:");

        assert_eq!(items, vec!["first item", "second item"]);
    }

    #[test]
    fn extract_done_items_stops_on_next_header() {
        let message = "done:\n- first\nNext Section:\n- should not be included";

        let items = extract_done_items(message, "done:");

        assert_eq!(items, vec!["first"]);
    }

    #[test]
    fn parse_commits_includes_only_records_with_done_items() {
        let input = concat!(
            "abc123\x1f1700000000\x1fsubject\n\ndone:\n- ship feature\n\x1e",
            "def456\x1f1700000050\x1fsubject without done marker\x1e"
        );

        let commits = parse_commits(input, "done:");

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].done_items, vec!["ship feature"]);
        assert_eq!(commits[0].date.timestamp(), 1700000000);
    }

    #[test]
    fn parse_commits_uses_epoch_for_invalid_timestamp() {
        let input = "abc123\x1fnot-a-number\x1fsubject\n\ndone:\n- keep\n\x1e";

        let commits = parse_commits(input, "done:");

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].date.timestamp(), 0);
    }
}
