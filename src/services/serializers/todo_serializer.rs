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
    let mut output = String::new();

    // --- Header ---
    writeln!(
        &mut output,
        "# TODO — {} v{}\n",
        list.project_name, list.project_version
    )
    .unwrap();

    writeln!(&mut output, "@created: {}", format_date(&list.created_at)).unwrap();
    writeln!(&mut output, "@modified: {}", format_date(&list.modified_at)).unwrap();

    let mut incomplete_tasks: Vec<_> = list.tasks.iter().filter(|t| !t.completed).collect();
    let mut completed_tasks: Vec<_> = list.tasks.iter().filter(|t| t.completed).collect();

    incomplete_tasks.sort_by_key(|t| t.created_at_time);
    completed_tasks.sort_by_key(|t| t.completed_at_time.unwrap_or(t.created_at_time));

    // Tasks
    writeln!(&mut output, "\n## Tasks\n").unwrap();
    for task in incomplete_tasks {
        write_task(&mut output, task);
        output += "\n";
    }

    // Completed
    if !completed_tasks.is_empty() {
        writeln!(&mut output, "\n## Completed\n").unwrap();
        for task in completed_tasks {
            write_task(&mut output, task);
            output += "\n";
        }
    }

    output
}

/// Parse a TODO.md string into a List
pub fn deserialize(content: &str) -> Result<List> {
    let mut lines = content.lines().peekable();

    let header = lines.next().context("Empty TODO file")?;
    let (project_name, project_version) = parse_header(header)?;

    let mut created_at = None;
    let mut modified_at = None;

    while let Some(line) = lines.peek() {
        if line.starts_with("@created:") {
            created_at = Some(parse_date_line(lines.next().unwrap())?);
        } else if line.starts_with("@modified:") {
            modified_at = Some(parse_date_line(lines.next().unwrap())?);
        } else if !line.trim().is_empty() {
            break;
        } else {
            lines.next();
        }
    }

    let created_at = created_at.context("Missing @created metadata")?;
    let modified_at = modified_at.context("Missing @modified metadata")?;

    let mut tasks = Vec::new();

    while let Some(line) = lines.next() {
        match line.trim() {
            "## Tasks" | "## Completed" => {
                // Parse tasks until next section or EOF
                let section_tasks = parse_tasks(&mut lines)?;
                tasks.extend(section_tasks);
            }
            _ => {}
        }
    }

    Ok(List {
        project_name,
        project_version,
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

fn parse_header(line: &str) -> Result<(String, Version)> {
    let line = line.trim_start_matches('#').trim();

    if !line.starts_with("TODO —") && !line.starts_with("TODO -") {
        anyhow::bail!("Invalid header format: expected 'TODO — [PROJECT] v[VERSION]'");
    }

    let line = line
        .trim_start_matches("TODO —")
        .trim_start_matches("TODO -")
        .trim();

    let version_start = line.rfind(" v").context("No version found in header")?;

    let project_name = line[..version_start].trim().to_string();
    let version_str = line[version_start + 2..].trim();

    let version = Version::parse(version_str).context("Failed to parse version in header")?;

    Ok((project_name, version))
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

    let created_at_time = created_at_time.context("Task missing @created timestamp")?;

    Ok(TaskMetadata {
        created_at_time,
        created_at_version,
        created_at_commit,
        completed_at_time,
        completed_at_version,
        completed_at_commit,
    })
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    let naive = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
        .context(format!("Failed to parse datetime: {}", s))?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

#[cfg(test)]
#[path = "../../../tests/services/serializers/todo_serializer_tests.rs"]
mod tests;
