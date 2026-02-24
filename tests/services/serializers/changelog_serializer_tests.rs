use super::*;
use crate::models::{
    changes::{Change, Log, Release},
    common::{Priority, Version},
};
use chrono::{TimeZone, Utc};
use std::collections::BTreeMap;

fn change(description: &str, priority: Priority, tags: &[&str], commit: Option<&str>) -> Change {
    Change {
        description: description.to_string(),
        priority,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        commit: commit.map(str::to_string),
        completed_at: Utc.with_ymd_and_hms(2026, 2, 20, 10, 30, 0).unwrap(),
    }
}

#[test]
fn to_markdown_renders_sections_tags_and_short_commits() {
    let mut by_priority = BTreeMap::new();
    by_priority.insert(
        Priority::High,
        vec![change(
            "Fix crash when history is empty",
            Priority::High,
            &["bug"],
            Some("1234567890abcdef"),
        )],
    );
    by_priority.insert(
        Priority::Low,
        vec![change("Polish docs", Priority::Low, &[], None)],
    );

    let release = Release {
        version: Version::new(1, 2, 3, false),
        date: Utc.with_ymd_and_hms(2026, 2, 21, 0, 0, 0).unwrap(),
        changes_by_priority: by_priority,
        changes_by_tag: BTreeMap::new(),
    };

    let log = Log {
        project_name: "tally".to_string(),
        releases: vec![release],
        generated_at: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
    };

    let markdown = to_markdown(&log);

    assert!(markdown.contains("# Changelog — tally"));
    assert!(markdown.contains("*Generated on 2026-02-23*"));
    assert!(markdown.contains("## 1.2.3 — 2026-02-21"));
    assert!(markdown.contains("### High Priority"));
    assert!(markdown.contains("### Minor Changes"));
    assert!(!markdown.contains("### Changes"));
    assert!(markdown.contains("- Fix crash when history is empty `bug` ([`1234567`])"));
    assert!(markdown.contains("- Polish docs"));
}

#[test]
fn serde_json_emits_valid_payload() {
    let release = Release {
        version: Version::new(0, 5, 0, false),
        date: Utc.with_ymd_and_hms(2026, 2, 2, 0, 0, 0).unwrap(),
        changes_by_priority: BTreeMap::new(),
        changes_by_tag: BTreeMap::new(),
    };

    let log = Log {
        project_name: "tally".to_string(),
        releases: vec![release],
        generated_at: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
    };

    let json = serde_json::to_string_pretty(&log).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["project_name"], "tally");
    assert_eq!(value["releases"].as_array().unwrap().len(), 1);
}
