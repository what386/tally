use crate::models::{
    common::{Priority, Version},
    tasks::{List, Task},
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fmt::Write;

struct TaskMetadata {
    created_at_time: DateTime<Utc>,
    created_at_version: Option<Version>,
    created_at_commit: Option<String>,
    completed_at_time: Option<DateTime<Utc>>,
    completed_at_version: Option<Version>,
    completed_at_commit: Option<String>,
}

pub fn serialize(list: &List) -> String {
    let mut output = render_header(list);
    output.push_str(&render_task_sections(list));
    output
}

pub fn serialize_preserving(list: &List, previous: Option<&str>) -> String {
    let Some(previous) = previous else {
        return serialize(list);
    };

    let mut output = String::new();
    output.push_str(&render_header(list));

    let preserved = preserved_todo_sections(previous);
    if !preserved.preface.trim().is_empty() {
        output.push_str(preserved.preface.trim());
        output.push_str("\n\n");
    }

    output.push_str(&render_task_sections(list));

    if !preserved.sections.is_empty() {
        if !output.ends_with("\n\n") {
            output.push('\n');
        }
        output.push_str(&preserved.sections.join("\n"));
        if !output.ends_with('\n') {
            output.push('\n');
        }
    }

    output
}

fn render_header(list: &List) -> String {
    let mut output = String::new();

    writeln!(&mut output, "# TODO — {}\n", list.project_name).unwrap();
    writeln!(&mut output, "@created: {}", format_date(&list.created_at)).unwrap();
    writeln!(&mut output, "@modified: {}", format_date(&list.modified_at)).unwrap();
    output.push('\n');

    output
}

fn render_task_sections(list: &List) -> String {
    let mut output = String::new();

    let mut incomplete_tasks: Vec<_> = list.tasks.iter().filter(|t| !t.completed).collect();
    let mut completed_tasks: Vec<_> = list.tasks.iter().filter(|t| t.completed).collect();

    incomplete_tasks.sort_by_key(|t| t.created_at_time);
    completed_tasks.sort_by_key(|t| t.completed_at_time.unwrap_or(t.created_at_time));

    writeln!(&mut output, "\n## Tasks\n").unwrap();
    for task in incomplete_tasks {
        write_task(&mut output, task);
        output += "\n";
    }

    if !completed_tasks.is_empty() {
        writeln!(&mut output, "\n## Completed\n").unwrap();
        for task in completed_tasks {
            write_task(&mut output, task);
            output += "\n";
        }
    }

    output
}

#[derive(Debug, Default)]
struct PreservedTodoSections {
    preface: String,
    sections: Vec<String>,
}

fn preserved_todo_sections(content: &str) -> PreservedTodoSections {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    if lines
        .first()
        .is_some_and(|line| line.trim_start().starts_with("# TODO"))
    {
        i += 1;
    }

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("@created:")
            || trimmed.starts_with("@modified:")
            || trimmed.is_empty()
        {
            i += 1;
            continue;
        }
        break;
    }

    let mut preface = Vec::new();
    while i < lines.len() && !lines[i].starts_with("## ") {
        preface.push(lines[i]);
        i += 1;
    }

    let mut sections = Vec::new();
    while i < lines.len() {
        let start = i;
        i += 1;
        while i < lines.len() && !lines[i].starts_with("## ") {
            i += 1;
        }

        let heading = lines[start].trim();
        if heading != "## Tasks" && heading != "## Completed" {
            sections.push(lines[start..i].join("\n").trim().to_string());
        }
    }

    PreservedTodoSections {
        preface: preface.join("\n"),
        sections: sections
            .into_iter()
            .filter(|section| !section.is_empty())
            .collect(),
    }
}

pub fn deserialize(content: &str) -> Result<List> {
    let lines: Vec<&str> = content.lines().collect();

    let header = lines.first().context("Empty TODO file")?;
    let project_name = parse_header(header)?;

    let mut created_at = None;
    let mut modified_at = None;
    let mut idx = 1;

    while let Some(line) = lines.get(idx) {
        if line.starts_with("@created:") {
            created_at = Some(parse_date_line(line)?);
        } else if line.starts_with("@modified:") {
            modified_at = Some(parse_date_line(line)?);
        } else if !line.trim().is_empty() {
            break;
        }
        idx += 1;
    }

    let created_at = created_at.context("Missing @created metadata")?;
    let modified_at = modified_at.context("Missing @modified metadata")?;

    let mut tasks = Vec::new();

    while idx < lines.len() {
        let line = lines[idx].trim();
        if line == "## Tasks" || line == "## Completed" {
            idx += 1;
            let start = idx;
            while idx < lines.len() && !lines[idx].starts_with("## ") {
                idx += 1;
            }
            tasks.extend(parse_tasks(lines[start..idx].iter().copied())?);
        } else {
            idx += 1;
        }
    }

    Ok(List {
        project_name,
        project_version: Version::new(0, 1, 0, false),
        created_at,
        modified_at,
        tasks,
    })
}

fn write_task(output: &mut String, task: &Task) {
    let checkbox = if task.completed { "x" } else { " " };

    let priority_str = match task.priority {
        Priority::High => " (high)",
        Priority::Medium => "",
        Priority::Low => " (low)",
    };

    let tags_str = if task.tags.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            task.tags
                .iter()
                .map(|t| format!("#{}", t))
                .collect::<Vec<_>>()
                .join(" ")
        )
    };

    writeln!(
        output,
        "- [{}] {}{}{}",
        checkbox, task.description, priority_str, tags_str
    )
    .unwrap();

    write_task_metadata(output, task);
}

fn write_task_metadata(output: &mut String, task: &Task) {
    writeln!(
        output,
        "      @created {}",
        format_datetime(&task.created_at_time)
    )
    .unwrap();

    if let Some(version) = &task.created_at_version {
        writeln!(output, "      @created_version {}", version).unwrap();
    }

    if let Some(commit) = &task.created_at_commit {
        writeln!(output, "      @created_commit {}", commit).unwrap();
    }

    if task.completed {
        if let Some(completed_time) = &task.completed_at_time {
            writeln!(
                output,
                "      @completed {}",
                format_datetime(completed_time)
            )
            .unwrap();
        }

        if let Some(version) = &task.completed_at_version {
            writeln!(output, "      @completed_version {}", version).unwrap();
        }

        if let Some(commit) = &task.completed_at_commit {
            writeln!(output, "      @completed_commit {}", commit).unwrap();
        }
    }
}

fn format_date(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

fn parse_header(line: &str) -> Result<String> {
    let line = line.trim_start_matches('#').trim();

    if !line.starts_with("TODO —") && !line.starts_with("TODO -") {
        anyhow::bail!("Invalid header format: expected 'TODO — [PROJECT]'");
    }

    let line = line
        .trim_start_matches("TODO —")
        .trim_start_matches("TODO -")
        .trim();

    let project_name = if let Some(version_start) = line.rfind(" v") {
        line[..version_start].trim().to_string()
    } else {
        line.to_string()
    };

    if project_name.is_empty() {
        anyhow::bail!("Missing project name in TODO header");
    }

    Ok(project_name)
}

fn parse_date_line(line: &str) -> Result<DateTime<Utc>> {
    let date_str = line
        .split(':')
        .nth(1)
        .context("Invalid date line format")?
        .trim();

    let naive_date =
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").context("Failed to parse date")?;
    let naive_datetime = naive_date.and_hms_opt(0, 0, 0).context("Invalid time")?;

    Ok(DateTime::<Utc>::from_naive_utc_and_offset(
        naive_datetime,
        Utc,
    ))
}

fn parse_tasks<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let mut current_task_lines = Vec::new();

    for line in lines {
        if line.starts_with("- [") {
            if !current_task_lines.is_empty() {
                tasks.push(parse_task(&current_task_lines)?);
                current_task_lines.clear();
            }
            current_task_lines.push(line.to_string());
        } else if !current_task_lines.is_empty() && !line.trim().is_empty() {
            current_task_lines.push(line.to_string());
        }
    }

    if !current_task_lines.is_empty() {
        tasks.push(parse_task(&current_task_lines)?);
    }

    Ok(tasks)
}

fn parse_task(lines: &[String]) -> Result<Task> {
    if lines.is_empty() {
        anyhow::bail!("Empty task lines");
    }

    let first_line = &lines[0];
    let completed = first_line.contains("[x]") || first_line.contains("[X]");

    let content = first_line
        .trim_start_matches("- [")
        .trim_start_matches(['x', 'X', ' '])
        .trim_start_matches(']')
        .trim();

    let (description, priority, tags) = parse_task_content(content)?;

    let metadata = parse_task_metadata(&lines[1..])?;

    Ok(Task {
        description,
        priority,
        tags,
        completed,
        created_at_time: metadata.created_at_time,
        created_at_version: metadata.created_at_version,
        created_at_commit: metadata.created_at_commit,
        completed_at_time: metadata.completed_at_time,
        completed_at_version: metadata.completed_at_version,
        completed_at_commit: metadata.completed_at_commit,
    })
}

fn parse_task_content(content: &str) -> Result<(String, Priority, Vec<String>)> {
    let mut tags = Vec::new();
    let mut description_parts = Vec::new();
    let mut priority = Priority::Medium;

    for part in content.split_whitespace() {
        if part.starts_with('#') {
            tags.push(part.trim_start_matches('#').to_string());
        } else if part == "(high)" {
            priority = Priority::High;
        } else if part == "(low)" {
            priority = Priority::Low;
        } else if part == "(medium)" {
            priority = Priority::Medium;
        } else {
            description_parts.push(part);
        }
    }

    let description = description_parts.join(" ");

    if description.is_empty() {
        anyhow::bail!("Task has no description");
    }

    Ok((description, priority, tags))
}

fn parse_task_metadata(lines: &[String]) -> Result<TaskMetadata> {
    let mut created_at_time = None;
    let mut created_at_version = None;
    let mut created_at_commit = None;
    let mut completed_at_time = None;
    let mut completed_at_version = None;
    let mut completed_at_commit = None;

    for line in lines {
        let line = line.trim();

        if let Some(value) = line.strip_prefix("@created ") {
            created_at_time = Some(parse_datetime(value.trim())?);
        } else if let Some(value) = line.strip_prefix("@created_version ") {
            created_at_version = Some(Version::parse(value.trim())?);
        } else if let Some(value) = line.strip_prefix("@created_commit ") {
            created_at_commit = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("@completed ") {
            completed_at_time = Some(parse_datetime(value.trim())?);
        } else if let Some(value) = line.strip_prefix("@completed_version ") {
            completed_at_version = Some(Version::parse(value.trim())?);
        } else if let Some(value) = line.strip_prefix("@completed_commit ") {
            completed_at_commit = Some(value.trim().to_string());
        }
    }

    Ok(TaskMetadata {
        created_at_time: created_at_time.context("Task missing @created metadata")?,
        created_at_version,
        created_at_commit,
        completed_at_time,
        completed_at_version,
        completed_at_commit,
    })
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    let naive = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M")
        .context("Failed to parse datetime")?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

#[cfg(test)]
mod tests {
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
        assert!(markdown.contains("# TODO — tally"));
        assert!(markdown.contains("## Tasks"));
        assert!(markdown.contains("## Completed"));

        let parsed = deserialize(&markdown).unwrap();
        assert_eq!(parsed.project_name, "tally");
        assert_eq!(parsed.project_version.to_string(), "0.1.0");
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
        let content = "# TODO - demo\n\n@created: 2026-02-20\n@modified: 2026-02-21\n\n## Tasks\n\n- [ ] keep parser compatibility\n      @created 2026-02-20 10:00\n";

        let parsed = deserialize(content).unwrap();
        assert_eq!(parsed.project_name, "demo");
        assert_eq!(parsed.project_version.to_string(), "0.1.0");
        assert_eq!(parsed.tasks.len(), 1);
        assert_eq!(parsed.tasks[0].description, "keep parser compatibility");
    }

    #[test]
    fn deserialize_requires_created_metadata() {
        let content = "# TODO — demo\n\n@modified: 2026-02-21\n\n## Tasks\n\n- [ ] missing metadata\n      @created 2026-02-20 10:00\n";

        let err = deserialize(content).unwrap_err();
        assert!(err.to_string().contains("Missing @created metadata"));
    }

    #[test]
    fn serialize_preserving_keeps_unmanaged_sections() {
        let mut list = List {
            project_name: "demo".to_string(),
            project_version: Version::new(0, 1, 0, false),
            created_at: Utc.with_ymd_and_hms(2026, 2, 20, 0, 0, 0).unwrap(),
            modified_at: Utc.with_ymd_and_hms(2026, 2, 21, 0, 0, 0).unwrap(),
            tasks: vec![],
        };
        list.tasks.push(Task {
            description: "managed task".to_string(),
            priority: Priority::Medium,
            tags: vec![],
            completed: false,
            created_at_time: Utc.with_ymd_and_hms(2026, 2, 20, 8, 15, 0).unwrap(),
            created_at_version: None,
            created_at_commit: None,
            completed_at_time: None,
            completed_at_version: None,
            completed_at_commit: None,
        });
        let previous = "# TODO — demo\n\n@created: 2026-02-20\n@modified: 2026-02-20\n\nProject notes stay here.\n\n## Tasks\n\n- [ ] old task\n      @created 2026-02-20 08:00\n\n## Notes\n\n- arbitrary markdown\n";

        let rendered = serialize_preserving(&list, Some(previous));

        assert!(rendered.contains("Project notes stay here."));
        assert!(rendered.contains("## Notes\n\n- arbitrary markdown"));
        assert!(rendered.contains("managed task"));
        assert!(!rendered.contains("old task"));
    }
}
