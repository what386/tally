use super::*;
use crate::models::{
    common::{Priority, Version},
    tasks::Task,
};
use chrono::{TimeZone, Utc};

fn task(
    description: &str,
    priority: Priority,
    tags: &[&str],
    completed: bool,
    completed_version: Option<Version>,
) -> Task {
    Task {
        description: description.to_string(),
        priority,
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
        completed,
        created_at_time: Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap(),
        created_at_version: None,
        created_at_commit: None,
        completed_at_time: completed.then(|| Utc.with_ymd_and_hms(2026, 4, 2, 12, 0, 0).unwrap()),
        completed_at_version: completed_version,
        completed_at_commit: None,
    }
}

#[test]
fn filter_tasks_done_only_returns_completed_tasks() {
    let tasks = vec![
        task("unfinished", Priority::Medium, &["feature"], false, None),
        task(
            "finished",
            Priority::High,
            &["feature"],
            true,
            Some(Version::new(0, 6, 0, false)),
        ),
    ];

    let filtered = filter_tasks(&tasks, None, None, true);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.description, "finished");
}

#[test]
fn filter_tasks_combines_filters_as_intersection() {
    let tasks = vec![
        task(
            "matching task",
            Priority::High,
            &["feature", "ux"],
            true,
            Some(Version::new(0, 6, 0, false)),
        ),
        task(
            "wrong priority",
            Priority::Medium,
            &["feature", "ux"],
            true,
            Some(Version::new(0, 6, 0, false)),
        ),
        task(
            "wrong tag",
            Priority::High,
            &["backend"],
            true,
            Some(Version::new(0, 6, 0, false)),
        ),
    ];
    let tags = vec!["feature".to_string(), "ux".to_string()];

    let filtered = filter_tasks(&tasks, Some(&tags), Some(&Priority::High), true);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.description, "matching task");
}
