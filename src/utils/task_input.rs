use crate::models::common::Priority;
use anyhow::Result;

pub struct ParsedTaskInput {
    pub description: String,
    pub priority: Priority,
    pub tags: Vec<String>,
}

pub fn parse_task_input(
    description: impl AsRef<str>,
    priority_override: Option<Priority>,
    tags_override: Option<Vec<String>>,
) -> Result<ParsedTaskInput> {
    let mut description_parts = Vec::new();
    let mut parsed_priority = None;
    let mut parsed_tags = Vec::new();

    for part in description.as_ref().split_whitespace() {
        if let Some(tag) = part.strip_prefix('#') {
            if !tag.is_empty() {
                parsed_tags.push(tag.to_string());
            }
            continue;
        }

        match parse_priority_marker(part) {
            Some(priority) => parsed_priority = Some(priority),
            None => description_parts.push(part),
        }
    }

    let description = description_parts.join(" ");

    if description.is_empty() {
        anyhow::bail!("Task has no description");
    }

    Ok(ParsedTaskInput {
        description,
        priority: priority_override
            .or(parsed_priority)
            .unwrap_or(Priority::Medium),
        tags: tags_override.unwrap_or(parsed_tags),
    })
}

fn parse_priority_marker(value: &str) -> Option<Priority> {
    match value.to_ascii_lowercase().as_str() {
        "(high)" => Some(Priority::High),
        "(medium)" => Some(Priority::Medium),
        "(low)" => Some(Priority::Low),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_priority_and_tags() {
        let input = parse_task_input(
            "Implement a new backend (high) #backend #improvement",
            None,
            None,
        )
        .unwrap();

        assert_eq!(input.description, "Implement a new backend");
        assert_eq!(input.priority, Priority::High);
        assert_eq!(input.tags, vec!["backend", "improvement"]);
    }

    #[test]
    fn explicit_flags_override_inline_metadata() {
        let input = parse_task_input(
            "Implement a new backend (low) #backend #improvement",
            Some(Priority::High),
            Some(vec!["api".to_string()]),
        )
        .unwrap();

        assert_eq!(input.description, "Implement a new backend");
        assert_eq!(input.priority, Priority::High);
        assert_eq!(input.tags, vec!["api"]);
    }

    #[test]
    fn defaults_to_medium_without_inline_or_flag_priority() {
        let input = parse_task_input("Update docs #docs", None, None).unwrap();

        assert_eq!(input.description, "Update docs");
        assert_eq!(input.priority, Priority::Medium);
        assert_eq!(input.tags, vec!["docs"]);
    }
}
