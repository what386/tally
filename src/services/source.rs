use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

const TODO_MARKER: &str = " TODO: ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTodo {
    pub path: String,
    pub line: usize,
    pub text: String,
}

impl SourceTodo {
    pub fn description(&self) -> String {
        format!("{}:{} - {}", self.path, self.line, self.text)
    }

    pub fn location(&self) -> String {
        format!("{}:{}", self.path, self.line)
    }
}

pub fn scan_project(root: &Path) -> Result<Vec<SourceTodo>> {
    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(root)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("failed to list tracked files");
    }

    let mut todos = Vec::new();
    let files = String::from_utf8(output.stdout)?;
    for rel_path in files.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if rel_path == "TODO.md" || rel_path == "CHANGELOG.md" {
            continue;
        }

        let file_path = root.join(rel_path);
        let bytes = match fs::read(&file_path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };

        let content = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => continue,
        };

        todos.extend(extract_todos_from_content(rel_path, &content));
    }

    Ok(todos)
}

pub fn extract_todos_from_content(path: &str, content: &str) -> Vec<SourceTodo> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let Some(marker_idx) = line.find(TODO_MARKER) else {
            i += 1;
            continue;
        };

        let indent_len = line.chars().take_while(|c| c.is_whitespace()).count();
        let indent = &line[..indent_len];
        let comment_prefix = &line[indent_len..marker_idx];
        if comment_prefix.trim().is_empty() {
            i += 1;
            continue;
        }

        let mut parts = Vec::new();
        let first_part = line[marker_idx + TODO_MARKER.len()..].trim();
        if !first_part.is_empty() {
            parts.push(first_part.to_string());
        }

        let base = format!("{}{}", indent, comment_prefix);

        let mut j = i + 1;
        while j < lines.len() {
            let next = lines[j];
            if !next.starts_with(&base) {
                break;
            }

            let continuation = next[base.len()..].trim();
            if continuation.is_empty() {
                break;
            }
            parts.push(continuation.to_string());
            j += 1;
        }

        let text = parts.join(" ").trim().to_string();
        if !text.is_empty() {
            result.push(SourceTodo {
                path: path.to_string(),
                line: i + 1,
                text,
            });
        }

        i = j;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::extract_todos_from_content;

    #[test]
    fn extracts_single_line_todo() {
        let content = "// TODO: fix parser\nlet x = 1;\n";
        let todos = extract_todos_from_content("src/main.rs", content);

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line, 1);
        assert_eq!(todos[0].text, "fix parser");
    }

    #[test]
    fn multiline_requires_same_indent_and_prefix() {
        let content = "// TODO: fix this\n// and this too\n   whatever\nfn main() {}\n";
        let todos = extract_todos_from_content("src/main.rs", content);

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "fix this and this too");
    }

    #[test]
    fn does_not_match_without_space_wrapped_marker() {
        let content = "//TODO: no match\n// TODO: yes\n";
        let todos = extract_todos_from_content("src/main.rs", content);

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "yes");
    }
}
