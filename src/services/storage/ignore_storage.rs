use std::fs;
use std::path::Path;

pub struct IgnoreStorage {
    rules: Vec<IgnoreRule>,
}

#[derive(Debug)]
enum IgnoreRule {
    /// Matches if the task description contains this substring (case-insensitive)
    Substring(String),
    /// Matches if the task description matches a glob pattern with * wildcards
    Glob(String),
    /// Matches if the task has this tag
    Tag(String),
}

impl IgnoreStorage {
    /// Load ignore rules from .tally/ignore. Returns empty rules if file doesn't exist.
    pub fn load(ignore_file: &Path) -> Self {
        let rules = match fs::read_to_string(ignore_file) {
            Ok(content) => content
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .filter_map(Self::parse_rule)
                .collect(),
            Err(_) => Vec::new(),
        };

        Self { rules }
    }

    fn parse_rule(line: &str) -> Option<IgnoreRule> {
        // Tag rule: #tagname
        if let Some(tag) = line.strip_prefix('#') {
            return Some(IgnoreRule::Tag(tag.to_lowercase()));
        }

        // Glob rule: contains *
        if line.contains('*') {
            return Some(IgnoreRule::Glob(line.to_lowercase()));
        }

        // Default: substring match
        Some(IgnoreRule::Substring(line.to_lowercase()))
    }

    /// Check if a task should be ignored
    pub fn is_ignored(&self, description: &str, tags: &[String]) -> bool {
        let desc_lower = description.to_lowercase();

        self.rules.iter().any(|rule| match rule {
            IgnoreRule::Substring(pattern) => desc_lower.contains(pattern.as_str()),
            IgnoreRule::Glob(pattern) => glob_match(pattern, &desc_lower),
            IgnoreRule::Tag(tag) => tags.iter().any(|t| t.to_lowercase() == *tag),
        })
    }

}

/// Simple glob matching supporting only `*` as a wildcard (matches any number of chars).
/// Both pattern and text should already be lowercased before calling.
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    // Edge case: pattern is just "*"
    if parts.len() == 1 {
        return text == pattern;
    }

    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        match text[pos..].find(part) {
            Some(idx) => {
                // First part must match at the start if pattern doesn't begin with *
                if i == 0 && idx != 0 {
                    return false;
                }
                pos += idx + part.len();
            }
            None => return false,
        }
    }

    // If pattern doesn't end with *, the text must end exactly where we stopped
    if !pattern.ends_with('*') {
        return pos == text.len();
    }

    true
}
