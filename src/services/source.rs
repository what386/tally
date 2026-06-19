use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

#[cfg(test)]
const DEFAULT_TODO_MARKER: &str = "TODO:";
#[cfg(test)]
const DEFAULT_DONE_MARKER: &str = "DONE:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceMarkerKind {
    Todo,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTodo {
    pub path: String,
    pub line: usize,
    pub text: String,
    pub kind: SourceMarkerKind,
}

impl SourceTodo {
    pub fn description(&self) -> String {
        format!("{}:{} - {}", self.path, self.line, self.text)
    }

    pub fn location(&self) -> String {
        format!("{}:{}", self.path, self.line)
    }
}

pub fn scan_project(
    root: &Path,
    todo_markers: &[String],
    done_markers: &[String],
) -> Result<Vec<SourceTodo>> {
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

        todos.extend(extract_todos_from_content_with_markers(
            rel_path,
            &content,
            todo_markers,
            done_markers,
        ));
    }

    Ok(todos)
}

#[cfg(test)]
pub fn extract_todos_from_content(path: &str, content: &str) -> Vec<SourceTodo> {
    let todo_markers = vec![DEFAULT_TODO_MARKER.to_string()];
    let done_markers = vec![DEFAULT_DONE_MARKER.to_string()];
    extract_todos_from_content_with_markers(path, content, &todo_markers, &done_markers)
}

pub fn extract_todos_from_content_with_markers(
    path: &str,
    content: &str,
    todo_markers: &[String],
    done_markers: &[String],
) -> Vec<SourceTodo> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let marker = find_marker(line, todo_markers, done_markers);
        let Some((marker_idx, marker_text, marker_kind)) = marker else {
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
        let first_part = line[marker_idx + marker_text.len()..].trim();
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
            if find_marker(next, todo_markers, done_markers).is_some() {
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
                kind: marker_kind,
            });
        }

        i = j;
    }

    result
}

fn find_marker<'a>(
    line: &str,
    todo_markers: &'a [String],
    done_markers: &'a [String],
) -> Option<(usize, &'a str, SourceMarkerKind)> {
    todo_markers
        .iter()
        .filter_map(|marker| {
            find_marker_index(line, marker)
                .map(|idx| (idx, marker.as_str(), SourceMarkerKind::Todo))
        })
        .chain(done_markers.iter().filter_map(|marker| {
            find_marker_index(line, marker)
                .map(|idx| (idx, marker.as_str(), SourceMarkerKind::Done))
        }))
        .min_by_key(|(idx, _, _)| *idx)
}

fn find_marker_index(line: &str, marker: &str) -> Option<usize> {
    let marker = marker.trim();
    if marker.is_empty() {
        return None;
    }

    line.match_indices(marker)
        .find(|(idx, _)| marker_has_leading_boundary(line, *idx))
        .map(|(idx, _)| idx)
}

fn marker_has_leading_boundary(line: &str, marker_idx: usize) -> bool {
    line[..marker_idx]
        .chars()
        .next_back()
        .map(char::is_whitespace)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{
        SourceMarkerKind, extract_todos_from_content, extract_todos_from_content_with_markers,
    };

    #[test]
    fn extracts_single_line_todo() {
        let content = "// TODO: fix parser\nlet x = 1;\n";
        let todos = extract_todos_from_content("src/main.rs", content);

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line, 1);
        assert_eq!(todos[0].text, "fix parser");
        assert_eq!(todos[0].kind, SourceMarkerKind::Todo);
    }

    #[test]
    fn multiline_requires_same_indent_and_prefix() {
        let content = "// TODO: fix this\n// and this too\n   whatever\nfn main() {}\n";
        let todos = extract_todos_from_content("src/main.rs", content);

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "fix this and this too");
        assert_eq!(todos[0].kind, SourceMarkerKind::Todo);
    }

    #[test]
    fn does_not_match_without_space_wrapped_marker() {
        let content = "//TODO: no match\n// TODO: yes\n";
        let todos = extract_todos_from_content("src/main.rs", content);

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "yes");
        assert_eq!(todos[0].kind, SourceMarkerKind::Todo);
    }

    #[test]
    fn extracts_done_marker() {
        let content = "# DONE: finish parser cleanup\n";
        let todos = extract_todos_from_content("src/main.rs", content);

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "finish parser cleanup");
        assert_eq!(todos[0].kind, SourceMarkerKind::Done);
    }

    #[test]
    fn extracts_configured_markers() {
        let content = "// FIXME: repair parser\n// SHIPPED: remove parser workaround\n";
        let todos = extract_todos_from_content_with_markers(
            "src/main.rs",
            content,
            &["FIXME:".to_string()],
            &["SHIPPED:".to_string()],
        );

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].text, "repair parser");
        assert_eq!(todos[0].kind, SourceMarkerKind::Todo);
        assert_eq!(todos[1].text, "remove parser workaround");
        assert_eq!(todos[1].kind, SourceMarkerKind::Done);
    }
}
