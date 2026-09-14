//! Pure unified-diff parsing and text comparison primitives.
//!
//! This crate deliberately has no filesystem, process, or agent-runtime
//! dependencies. Consumers that need git or file IO should provide those
//! concerns in their owning crate.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// A parsed unified-diff hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub file_path: String,
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub header: String,
    pub content: String,
}

/// Parse raw unified diff output into structured hunks.
pub fn parse_unified_diff(diff: &str) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_file = "";

    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            if let Some(path) = rest.split_whitespace().nth(1) {
                current_file = path.trim_start_matches("b/");
            }
        } else if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = path;
        } else if line.starts_with("@@ ") {
            let (old_start, old_lines, new_start, new_lines) = parse_hunk_header(line);
            hunks.push(DiffHunk {
                file_path: current_file.to_string(),
                old_start,
                old_lines,
                new_start,
                new_lines,
                header: line.to_string(),
                content: String::new(),
            });
        } else if let Some(last) = hunks.last_mut() {
            if !last.content.is_empty() {
                last.content.push('\n');
            }
            last.content.push_str(line);
        }
    }

    hunks
}

/// Merge adjacent or overlapping hunks for the same file.
pub fn merge_hunks(mut hunks: Vec<DiffHunk>) -> Vec<DiffHunk> {
    let mut merged: Vec<DiffHunk> = Vec::with_capacity(hunks.len());
    for hunk in hunks.drain(..) {
        let Some(previous) = merged.last_mut() else {
            merged.push(hunk);
            continue;
        };
        let old_end = previous.old_start.saturating_add(previous.old_lines);
        let new_end = previous.new_start.saturating_add(previous.new_lines);
        let adjacent = previous.file_path == hunk.file_path
            && hunk.old_start <= old_end
            && hunk.new_start <= new_end;
        if !adjacent {
            merged.push(hunk);
            continue;
        }
        previous.old_lines = hunk
            .old_start
            .saturating_add(hunk.old_lines)
            .saturating_sub(previous.old_start);
        previous.new_lines = hunk
            .new_start
            .saturating_add(hunk.new_lines)
            .saturating_sub(previous.new_start);
        if !previous.content.is_empty() && !hunk.content.is_empty() {
            previous.content.push('\n');
        }
        previous.content.push_str(&hunk.content);
        previous.header.push_str("\n");
        previous.header.push_str(&hunk.header);
    }
    merged
}

/// Render a text comparison as a standard unified diff.
pub fn unified_text_diff(old: &str, new: &str, context: usize) -> String {
    similar::TextDiff::from_lines(old, new)
        .unified_diff()
        .context_radius(context)
        .header("a", "b")
        .to_string()
}

fn parse_hunk_header(header: &str) -> (usize, usize, usize, usize) {
    let mut bounds = (1, 1, 1, 1);
    if let Some(inside) = header.strip_prefix("@@ -").and_then(|s| s.split(" @@").next()) {
        let mut parts = inside.split(" +");
        if let Some(old) = parts.next() {
            let values: Vec<_> = old.split(',').collect();
            bounds.0 = values.first().and_then(|v| v.parse().ok()).unwrap_or(1);
            bounds.1 = values.get(1).and_then(|v| v.parse().ok()).unwrap_or(1);
        }
        if let Some(new) = parts.next() {
            let values: Vec<_> = new.split(',').collect();
            bounds.2 = values.first().and_then(|v| v.parse().ok()).unwrap_or(1);
            bounds.3 = values.get(1).and_then(|v| v.parse().ok()).unwrap_or(1);
        }
    }
    bounds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_merges_adjacent_hunks() {
        let diff = "diff --git a/a.txt b/a.txt\n+++ b/a.txt\n@@ -1,1 +1,2 @@\n-old\n+new\n@@ -2,1 +2,1 @@\n context\n";
        let hunks = parse_unified_diff(diff);
        assert_eq!(hunks.len(), 2);
        let merged = merge_hunks(hunks);
        assert_eq!(merged.len(), 1);
        assert!(merged[0].content.contains("+new"));
    }

    #[test]
    fn renders_text_diff_without_runtime_dependencies() {
        let rendered = unified_text_diff("one\n", "two\n", 3);
        assert!(rendered.contains("-one"));
        assert!(rendered.contains("+two"));
    }
}
