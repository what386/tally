use super::*;
use crate::models::common::Priority;
use chrono::{TimeZone, Utc};

fn make_task(
    description: &str,
    commit: Option<&str>,
    version: Option<Version>,
    completed_at: chrono::DateTime<Utc>,
) -> Task {
    Task {
        description: description.to_string(),
        priority: Priority::High,
        tags: vec![],
        completed: true,
        created_at_time: completed_at,
        created_at_version: None,
        created_at_commit: None,
        completed_at_time: Some(completed_at),
        completed_at_version: version,
        completed_at_commit: commit.map(str::to_string),
    }
}

fn make_entry(
    description: &str,
    commit: Option<&str>,
    version: Option<Version>,
    completed_at: chrono::DateTime<Utc>,
) -> HistoryEntry {
    HistoryEntry {
        change: Change {
            description: description.to_string(),
            priority: Priority::High,
            tags: vec![],
            commit: commit.map(str::to_string),
            completed_at,
        },
        version,
    }
}

#[test]
fn record_all_dedupes_when_commit_matches() {
    let version = Version::new(0, 5, 0, false);
    let first_time = Utc.with_ymd_and_hms(2026, 2, 2, 19, 19, 32).unwrap();
    let second_time = Utc.with_ymd_and_hms(2026, 2, 2, 19, 19, 0).unwrap();

    let mut storage = HistoryStorage {
        entries: vec![make_entry(
            "github doing!",
            Some("9ba2c4a"),
            Some(version.clone()),
            first_time,
        )],
        history_file: std::env::temp_dir().join("tally-history-record-all-commit.json"),
    };

    let task = make_task("github doing!", Some("9ba2c4a"), Some(version), second_time);
    storage.record_all(&[&task]).unwrap();

    assert_eq!(storage.entries.len(), 1);
}

#[test]
fn record_all_dedupes_when_description_and_version_match() {
    let first_time = Utc.with_ymd_and_hms(2026, 2, 2, 2, 15, 14).unwrap();
    let second_time = Utc.with_ymd_and_hms(2026, 2, 2, 2, 15, 0).unwrap();

    let mut storage = HistoryStorage {
        entries: vec![make_entry("config support", None, None, first_time)],
        history_file: std::env::temp_dir().join("tally-history-record-all-desc-version.json"),
    };

    let task = make_task("config support", None, None, second_time);
    storage.record_all(&[&task]).unwrap();

    assert_eq!(storage.entries.len(), 1);
}

#[test]
fn record_all_keeps_distinct_versions() {
    let old_version = Version::new(0, 4, 0, false);
    let new_version = Version::new(0, 5, 0, false);
    let timestamp = Utc.with_ymd_and_hms(2026, 2, 2, 2, 15, 0).unwrap();

    let mut storage = HistoryStorage {
        entries: vec![make_entry(
            "release notes update",
            None,
            Some(old_version),
            timestamp,
        )],
        history_file: std::env::temp_dir().join("tally-history-record-all-versions.json"),
    };

    let task = make_task("release notes update", None, Some(new_version), timestamp);
    storage.record_all(&[&task]).unwrap();

    assert_eq!(storage.entries.len(), 2);
}
