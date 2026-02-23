use super::*;
use crate::models::{
    common::{Priority, Version},
    tasks::{List, Task},
};
use chrono::{TimeZone, Utc};

#[test]
fn serialize_deserialize_round_trip_preserves_task_metadata() {
    let mut list = List {
        project_name: "tally".to_string(),
        project_version: Version::new(0, 5, 0, false),
        created_at: Utc.with_ymd_and_hms(2026, 2, 20, 0, 0, 0).unwrap(),
        modified_at: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
        tasks: vec![],
    };

    list.tasks.push(Task {
        description: "config support".to_string(),
        priority: Priority::Low,
        tags: vec!["feature".to_string()],
        completed: false,
        created_at_time: Utc.with_ymd_and_hms(2026, 2, 20, 8, 15, 0).unwrap(),
        created_at_version: Some(Version::new(0, 4, 0, false)),
        created_at_commit: Some("abc1234".to_string()),
        completed_at_time: None,
        completed_at_version: None,
        completed_at_commit: None,
    });

    list.tasks.push(Task {
        description: "fix duplication in history".to_string(),
        priority: Priority::High,
        tags: vec![],
        completed: true,
        created_at_time: Utc.with_ymd_and_hms(2026, 2, 19, 9, 0, 0).unwrap(),
        created_at_version: None,
        created_at_commit: None,
        completed_at_time: Some(Utc.with_ymd_and_hms(2026, 2, 21, 9, 45, 0).unwrap()),
        completed_at_version: Some(Version::new(0, 3, 2, false)),
        completed_at_commit: Some("a556fb5".to_string()),
    });

    let markdown = serialize(&list);
    assert!(markdown.contains("# TODO — tally v0.5.0"));
    assert!(markdown.contains("## Tasks"));
    assert!(markdown.contains("## Completed"));

    let parsed = deserialize(&markdown).unwrap();
    assert_eq!(parsed.project_name, "tally");
    assert_eq!(parsed.project_version.to_string(), "0.5.0");
    assert_eq!(parsed.tasks.len(), 2);

    let config_task = parsed
        .tasks
        .iter()
        .find(|task| task.description == "config support")
        .unwrap();
    assert!(!config_task.completed);
    assert_eq!(config_task.priority, Priority::Low);
    assert_eq!(config_task.tags, vec!["feature"]);
    assert_eq!(config_task.created_at_commit.as_deref(), Some("abc1234"));
    assert_eq!(
        config_task
            .created_at_version
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("0.4.0")
    );

    let fixed_task = parsed
        .tasks
        .iter()
        .find(|task| task.description == "fix duplication in history")
        .unwrap();
    assert!(fixed_task.completed);
    assert_eq!(fixed_task.priority, Priority::High);
    assert_eq!(fixed_task.completed_at_commit.as_deref(), Some("a556fb5"));
    assert_eq!(
        fixed_task
            .completed_at_version
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("0.3.2")
    );
}

#[test]
fn deserialize_accepts_ascii_hyphen_header() {
    let content = "# TODO - demo v1.2.3\n\n@created: 2026-02-20\n@modified: 2026-02-21\n\n## Tasks\n\n- [ ] keep parser compatibility\n      @created 2026-02-20 10:00\n";

    let parsed = deserialize(content).unwrap();
    assert_eq!(parsed.project_name, "demo");
    assert_eq!(parsed.project_version.to_string(), "1.2.3");
    assert_eq!(parsed.tasks.len(), 1);
    assert_eq!(parsed.tasks[0].description, "keep parser compatibility");
}

#[test]
fn deserialize_requires_created_metadata() {
    let content = "# TODO — demo v1.2.3\n\n@modified: 2026-02-21\n\n## Tasks\n\n- [ ] missing metadata\n      @created 2026-02-20 10:00\n";

    let err = deserialize(content).unwrap_err();
    assert!(err.to_string().contains("Missing @created metadata"));
}
