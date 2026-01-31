use crate::models::{
    changes::{Log, Release},
    common::Priority,
};

/// Convert Changelog to markdown format
pub fn to_markdown(changelog: &Log) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Changelog — {}\n\n", changelog.project_name));
    output.push_str(&format!(
        "*Generated on {}*\n\n",
        changelog.generated_at.format("%Y-%m-%d")
    ));

    for release in &changelog.releases {
        output.push_str(&release_to_markdown(release));
        output.push('\n');
    }

    output
}

/// Convert Changelog to JSON format
pub fn to_json(changelog: &Log) -> serde_json::Result<String> {
    serde_json::to_string_pretty(changelog)
}

fn release_to_markdown(release: &Release) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "## {} — {}\n\n",
        release.version,
        release.date.format("%Y-%m-%d")
    ));

    for (priority, section_name) in [
        (Priority::High, "🔴 High Priority"),
        (Priority::Medium, "📝 Changes"),
        (Priority::Low, "🔹 Minor Changes"),
    ] {
        if let Some(changes) = release.changes_by_priority.get(&priority)
            && !changes.is_empty() {
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
