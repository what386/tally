use std::fmt::Write;
use chrono::{DateTime, Utc};
use crate::models::{
    enums::Priority, tasks::{List, Task}, version::Version
};

impl List {
    /// Serialize the List to TODO.md format
    pub fn to_todo_string(&self) -> String {
        let mut output = String::new();

        // Header
        writeln!(
            &mut output,
            "# TODO — {} v{}",
            self.project_name,
            self.project_version
        ).unwrap();

        writeln!(
            &mut output,
            "@created: {}",
            format_date(&self.created_at)
        ).unwrap();

        writeln!(
            &mut output,
            "@modified: {}",
            format_date(&self.modified_at)
        ).unwrap();

        // Tasks section
        writeln!(&mut output, "\n## Tasks").unwrap();

        for task in &self.tasks {
            write_task(&mut output, task);
        }

        output
    }

    /// Parse a TODO.md string into a List
    pub fn from_todo_string(content: &str) -> anyhow::Result<Self> {
        let mut lines = content.lines().peekable();

        // Parse header
        let header = lines.next()
            .ok_or_else(|| anyhow::anyhow!("Empty TODO file"))?;

        let (project_name, project_version) = parse_header(header)?;

        // Parse metadata
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

        let created_at = created_at
            .ok_or_else(|| anyhow::anyhow!("Missing @created metadata"))?;
        let modified_at = modified_at
            .ok_or_else(|| anyhow::anyhow!("Missing @modified metadata"))?;

        // Skip to tasks section
        while let Some(line) = lines.next() {
            if line.starts_with("## Tasks") {
                break;
            }
        }

        // Parse tasks
        let mut tasks = Vec::new();
        let mut current_task_lines = Vec::new();

        for line in lines {
            if line.starts_with("- [") {
                // Process previous task if any
                if !current_task_lines.is_empty() {
                    tasks.push(parse_task(&current_task_lines)?);
                    current_task_lines.clear();
                }
                current_task_lines.push(line.to_string());
            } else if !current_task_lines.is_empty() {
                // Continuation line (metadata)
                current_task_lines.push(line.to_string());
            }
        }

        // Process last task
        if !current_task_lines.is_empty() {
            tasks.push(parse_task(&current_task_lines)?);
        }

        Ok(List {
            project_name,
            project_version,
            created_at,
            modified_at,
            tasks,
        })
    }
}

fn write_task(output: &mut String, task: &Task) {
    let checkbox = if task.completed { "x" } else { " " };

    // Build main task line
    let priority_str = match task.priority {
        Priority::High => " (high)",
        Priority::Medium => "",
        Priority::Low => " (low)",
    };

    let tags_str = if task.tags.is_empty() {
        String::new()
    } else {
        format!(" {}", task.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "))
    };

    writeln!(
        output,
        "- [{}] {}{}{}",
        checkbox,
        task.description,
        priority_str,
        tags_str
    ).unwrap();

    // Write metadata (indented with 6 spaces)
    writeln!(
        output,
        "      @created {}",
        format_datetime(&task.created_at_time)
    ).unwrap();

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
            ).unwrap();
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

fn parse_header(line: &str) -> anyhow::Result<(String, Version)> {
    // Expected: "# TODO — [PROJECT_NAME] v[VERSION]"
    let line = line.trim_start_matches('#').trim();

    if !line.starts_with("TODO —") && !line.starts_with("TODO -") {
        anyhow::bail!("Invalid header format");
    }

    let line = line.trim_start_matches("TODO —").trim_start_matches("TODO -").trim();

    // Find the last 'v' which should be the version
    let version_start = line.rfind(" v")
        .ok_or_else(|| anyhow::anyhow!("No version found in header"))?;

    let project_name = line[..version_start].trim().to_string();
    let version_str = line[version_start + 2..].trim();

    let version = Version::parse(version_str)?;

    Ok((project_name, version))
}

fn parse_date_line(line: &str) -> anyhow::Result<DateTime<Utc>> {
    // Expected: "@created: 2026-01-30" or "@modified: 2026-01-30"
    let date_str = line.split(':')
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Invalid date line format"))?
        .trim();

    let naive_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let naive_datetime = naive_date.and_hms_opt(0, 0, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;

    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc))
}

fn parse_task(lines: &[String]) -> anyhow::Result<Task> {
    if lines.is_empty() {
        anyhow::bail!("Empty task lines");
    }

    // Parse first line: "- [x] Description (high) #tag1 #tag2"
    let first_line = &lines[0];

    let completed = first_line.contains("[x]") || first_line.contains("[X]");

    // Remove checkbox
    let content = first_line
        .trim_start_matches("- [")
        .trim_start_matches(|c| c == 'x' || c == 'X' || c == ' ')
        .trim_start_matches(']')
        .trim();

    // Extract tags (anything starting with #)
    let mut tags = Vec::new();
    let mut description = String::new();
    let mut priority = Priority::Medium;

    let parts: Vec<&str> = content.split_whitespace().collect();
    let mut i = 0;

    while i < parts.len() {
        let part = parts[i];

        if part.starts_with('#') {
            tags.push(part.trim_start_matches('#').to_string());
        } else if part == "(high)" {
            priority = Priority::High;
        } else if part == "(low)" {
            priority = Priority::Low;
        } else if part == "(medium)" {
            priority = Priority::Medium;
        } else {
            if !description.is_empty() {
                description.push(' ');
            }
            description.push_str(part);
        }

        i += 1;
    }

    // Parse metadata lines
    let mut created_at_time = None;
    let mut created_at_version = None;
    let mut created_at_commit = None;
    let mut completed_at_time = None;
    let mut completed_at_version = None;
    let mut completed_at_commit = None;

    for line in lines.iter().skip(1) {
        let line = line.trim();

        if line.starts_with("@created ") {
            let datetime_str = line.trim_start_matches("@created ").trim();
            created_at_time = Some(parse_datetime(datetime_str)?);
        } else if line.starts_with("@created_version ") {
            let version_str = line.trim_start_matches("@created_version ").trim();
            created_at_version = Some(Version::parse(version_str)?);
        } else if line.starts_with("@created_commit ") {
            created_at_commit = Some(line.trim_start_matches("@created_commit ").trim().to_string());
        } else if line.starts_with("@completed ") {
            let datetime_str = line.trim_start_matches("@completed ").trim();
            completed_at_time = Some(parse_datetime(datetime_str)?);
        } else if line.starts_with("@completed_version ") {
            let version_str = line.trim_start_matches("@completed_version ").trim();
            completed_at_version = Some(Version::parse(version_str)?);
        } else if line.starts_with("@completed_commit ") {
            completed_at_commit = Some(line.trim_start_matches("@completed_commit ").trim().to_string());
        }
    }

    let created_at_time = created_at_time
        .ok_or_else(|| anyhow::anyhow!("Task missing @created timestamp"))?;

    Ok(Task {
        description,
        priority,
        tags,
        completed,
        created_at_time,
        created_at_version,
        created_at_commit,
        completed_at_time,
        completed_at_version,
        completed_at_commit,
    })
}

fn parse_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    let naive = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}
