use crate::models::{
    changes::{Change, Log, Release},
    common::{Priority, Version},
};
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::BTreeMap;

pub fn to_markdown(changelog: &Log) -> String {
    let mut output = String::new();
    output.push_str(&render_changelog_header(changelog));

    for release in &changelog.releases {
        output.push_str(&release_to_markdown(release));
        output.push('\n');
    }

    output
}

pub fn to_markdown_preserving(changelog: &Log, previous: Option<&str>) -> String {
    let Some(previous) = previous else {
        return to_markdown(changelog);
    };

    let preserved = preserved_changelog_sections(previous);
    let mut output = String::new();
    output.push_str(&render_changelog_header(changelog));

    if !preserved.intro.trim().is_empty() {
        output.push_str(preserved.intro.trim());
        output.push_str("\n\n");
    }

    for release in &changelog.releases {
        output.push_str(&release_to_markdown(release));
        if let Some(extra) = preserved.release_extras.get(&release.version)
            && !extra.trim().is_empty()
        {
            output.push_str(extra.trim());
            output.push_str("\n\n");
        }
        output.push('\n');
    }

    if !preserved.top_level_sections.is_empty() {
        output.push_str(&preserved.top_level_sections.join("\n\n"));
        output.push('\n');
    }

    output
}

fn render_changelog_header(changelog: &Log) -> String {
    format!(
        "# Changelog — {}\n\n*Generated on {}*\n\n",
        changelog.project_name,
        changelog.generated_at.format("%Y-%m-%d")
    )
}

pub fn from_markdown(content: &str) -> Result<Log> {
    let mut project_name = "Untitled".to_string();
    let mut generated_at = Utc::now();
    let mut releases: Vec<Release> = Vec::new();

    let mut current_version: Option<Version> = None;
    let mut current_date = Utc::now();
    let mut current_priority = None;
    let mut current_changes: Vec<Change> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("# Changelog —") {
            let name = rest.trim();
            if !name.is_empty() {
                project_name = name.to_string();
            }
            continue;
        }

        if let Some(date_str) = trimmed
            .strip_prefix("*Generated on ")
            .and_then(|s| s.strip_suffix('*'))
        {
            if let Some(dt) = parse_date(date_str.trim()) {
                generated_at = dt;
            }
            continue;
        }

        if let Some(release) = parse_release_header(trimmed) {
            if let Some(version) = current_version.take() {
                let refs: Vec<&Change> = current_changes.iter().collect();
                releases.push(Release::from_changes(version, current_date, refs));
                current_changes.clear();
            }
            current_version = Some(release.0);
            current_date = release.1;
            current_priority = None;
            continue;
        }

        if let Some(priority) = parse_priority_header(trimmed) {
            current_priority = Some(priority);
            continue;
        }

        if trimmed.starts_with("### ") {
            current_priority = None;
            continue;
        }

        if let Some(priority) = current_priority
            && let Some(change) = parse_bullet_change(trimmed, priority)
            && current_version.is_some()
        {
            current_changes.push(change);
        }
    }

    if let Some(version) = current_version.take() {
        let refs: Vec<&Change> = current_changes.iter().collect();
        releases.push(Release::from_changes(version, current_date, refs));
    }

    Ok(Log {
        project_name,
        releases,
        generated_at,
    })
}

#[derive(Debug, Default)]
struct PreservedChangelogSections {
    intro: String,
    top_level_sections: Vec<String>,
    release_extras: BTreeMap<Version, String>,
}

fn preserved_changelog_sections(content: &str) -> PreservedChangelogSections {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    if lines
        .first()
        .is_some_and(|line| line.trim_start().starts_with("# Changelog"))
    {
        i += 1;
    }

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("*Generated on ") || trimmed.is_empty() {
            i += 1;
            continue;
        }
        break;
    }

    let mut intro = Vec::new();
    while i < lines.len() && !lines[i].starts_with("## ") {
        intro.push(lines[i]);
        i += 1;
    }

    let mut top_level_sections = Vec::new();
    let mut release_extras = BTreeMap::new();
    while i < lines.len() {
        let start = i;
        i += 1;
        while i < lines.len() && !lines[i].starts_with("## ") {
            i += 1;
        }

        let block = &lines[start..i];
        if let Some((version, _)) = parse_release_header(block[0].trim()) {
            let extra = preserved_release_extra(block);
            if !extra.trim().is_empty() {
                release_extras.insert(version, extra);
            }
        } else {
            let section = block.join("\n").trim().to_string();
            if !section.is_empty() {
                top_level_sections.push(section);
            }
        }
    }

    PreservedChangelogSections {
        intro: intro.join("\n"),
        top_level_sections,
        release_extras,
    }
}

fn preserved_release_extra(block: &[&str]) -> String {
    let mut extra = Vec::new();
    let mut i = 1;

    while i < block.len() {
        if parse_priority_header(block[i].trim()).is_some() {
            i += 1;
            while i < block.len() && !block[i].starts_with("### ") {
                i += 1;
            }
            continue;
        }

        let start = i;
        i += 1;
        while i < block.len()
            && !block[i].starts_with("### ")
            && parse_priority_header(block[i].trim()).is_none()
        {
            i += 1;
        }

        let section = block[start..i].join("\n").trim().to_string();
        if !section.is_empty() {
            extra.push(section);
        }
    }

    extra.join("\n\n")
}

fn release_to_markdown(release: &Release) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "## {} — {}\n\n",
        release.version,
        release.date.format("%Y-%m-%d")
    ));

    for (priority, section_name) in [
        (Priority::High, "High Priority"),
        (Priority::Medium, "Changes"),
        (Priority::Low, "Minor Changes"),
    ] {
        if let Some(changes) = release.changes_by_priority.get(&priority)
            && !changes.is_empty()
        {
            output.push_str(&format!("### {}\n\n", section_name));

            for change in changes {
                let tags = if change.tags.is_empty() {
                    String::new()
                } else {
                    format!(" `{}`", change.tags.join("`, `"))
                };

                let commit = change
                    .commit
                    .as_ref()
                    .map(|c| format!(" ([`{}`])", &c[..7.min(c.len())]))
                    .unwrap_or_default();

                output.push_str(&format!("- {}{}{}\n", change.description, tags, commit));
            }

            output.push('\n');
        }
    }

    output
}

fn parse_release_header(line: &str) -> Option<(Version, DateTime<Utc>)> {
    let body = line.strip_prefix("## ")?;
    let mut parts = body.split('—');
    let version_part = parts.next()?.trim();
    let date_part = parts.next().map(str::trim).unwrap_or_default();

    let version = Version::parse(version_part).ok()?;
    let date = parse_date(date_part).unwrap_or_else(Utc::now);
    Some((version, date))
}

fn parse_priority_header(line: &str) -> Option<Priority> {
    match line {
        "### High Priority" => Some(Priority::High),
        "### Changes" => Some(Priority::Medium),
        "### Minor Changes" => Some(Priority::Low),
        _ => None,
    }
}

fn parse_bullet_change(line: &str, priority: Priority) -> Option<Change> {
    let body = line.strip_prefix("- ")?.trim();
    if body.is_empty() {
        return None;
    }

    let (without_commit, commit) = extract_commit(body);
    let (description, tags) = extract_tags(without_commit);

    Some(Change {
        description,
        priority,
        tags,
        commit,
        completed_at: Utc::now(),
    })
}

fn extract_commit(text: &str) -> (&str, Option<String>) {
    if let Some(start) = text.rfind(" ([`")
        && let Some(end) = text[start + 4..].find("`])")
    {
        let hash = &text[start + 4..start + 4 + end];
        return (text[..start].trim_end(), Some(hash.to_string()));
    }
    (text, None)
}

fn extract_tags(text: &str) -> (String, Vec<String>) {
    let mut description = text.trim().to_string();
    let mut tags = Vec::new();

    if description.ends_with('`')
        && let Some(start) = description[..description.len() - 1].rfind(" `")
    {
        let candidate = &description[start + 2..description.len() - 1];
        if !candidate.is_empty() && candidate.split(", ").all(|t| !t.is_empty()) {
            tags = candidate.split(", ").map(|s| s.to_string()).collect();
            description = description[..start].trim_end().to_string();
        }
    }

    (description, tags)
}

fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let date = NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?;
    let dt = date.and_hms_opt(0, 0, 0)?;
    Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}

pub fn empty_log(project_name: &str) -> Log {
    Log {
        project_name: project_name.to_string(),
        releases: Vec::new(),
        generated_at: Utc::now(),
    }
}

pub fn normalize(log: &mut Log) {
    let mut merged: BTreeMap<Version, Vec<Change>> = BTreeMap::new();
    let mut date_map: BTreeMap<Version, DateTime<Utc>> = BTreeMap::new();

    for release in &log.releases {
        let mut changes = Vec::new();
        for group in release.changes_by_priority.values() {
            for change in group {
                changes.push(change.clone());
            }
        }

        merged
            .entry(release.version.clone())
            .or_default()
            .extend(changes);

        let entry = date_map
            .entry(release.version.clone())
            .or_insert(release.date);
        if release.date > *entry {
            *entry = release.date;
        }
    }

    log.releases = merged
        .into_iter()
        .map(|(version, changes)| {
            let refs: Vec<&Change> = changes.iter().collect();
            Release::from_changes(
                version.clone(),
                *date_map.get(&version).unwrap_or(&Utc::now()),
                refs,
            )
        })
        .collect();

    log.releases.sort_by(|a, b| b.version.cmp(&a.version));
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn change(
        description: &str,
        priority: Priority,
        tags: &[&str],
        commit: Option<&str>,
    ) -> Change {
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

    #[test]
    fn to_markdown_preserving_keeps_custom_release_notes_and_sections() {
        let version = Version::new(1, 2, 3, false);
        let release = Release::from_changes(
            version,
            Utc.with_ymd_and_hms(2026, 2, 21, 0, 0, 0).unwrap(),
            vec![&change("Managed change", Priority::Medium, &[], None)],
        );
        let log = Log {
            project_name: "tally".to_string(),
            releases: vec![release],
            generated_at: Utc.with_ymd_and_hms(2026, 2, 23, 0, 0, 0).unwrap(),
        };
        let previous = "# Changelog — tally\n\n*Generated on 2026-02-22*\n\nIntro note.\n\n## 1.2.3 — 2026-02-21\n\n### Changes\n\n- Old generated change\n\n### Notes\n\nKeep this note.\n\n## Appendix\n\nKeep this too.\n";

        let rendered = to_markdown_preserving(&log, Some(previous));

        assert!(rendered.contains("Intro note."));
        assert!(rendered.contains("### Notes\n\nKeep this note."));
        assert!(rendered.contains("## Appendix\n\nKeep this too."));
        assert!(rendered.contains("- Managed change"));
        assert!(!rendered.contains("Old generated change"));
    }
}
