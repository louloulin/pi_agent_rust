//! Git porcelain and reference parsing primitives.
//!
//! This crate intentionally does not execute Git. Callers that own runtime and
//! trust policy can feed command output into these deterministic parsers.

use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatusSummary {
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub deleted: usize,
    pub total: usize,
}

pub fn summarize_porcelain(output: &str) -> StatusSummary {
    let mut summary = StatusSummary::default();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        summary.total += 1;
        let bytes = line.as_bytes();
        let x = bytes.first().copied().unwrap_or(b' ');
        let y = bytes.get(1).copied().unwrap_or(b' ');
        if x == b'?' && y == b'?' {
            summary.untracked += 1;
            continue;
        }
        if x != b' ' { summary.staged += 1; }
        if y != b' ' { summary.unstaged += 1; }
        if x == b'D' || y == b'D' { summary.deleted += 1; }
    }
    summary
}

pub fn porcelain_paths(output: &str) -> Vec<String> {
    let mut paths = BTreeSet::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let raw = line.get(3..).unwrap_or(line).trim();
        let path = raw.rsplit(" -> ").next().unwrap_or(raw).trim_matches('"').trim();
        if !path.is_empty() { paths.insert(path.to_owned()); }
    }
    paths.into_iter().collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadRef {
    Branch(String),
    Detached(String),
}

pub fn parse_head(head: &str) -> Option<HeadRef> {
    let head = head.trim();
    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
        return non_empty(branch).map(|value| HeadRef::Branch(value.to_owned()));
    }
    let oid = head.strip_suffix('\n').unwrap_or(head);
    if oid.is_empty() { return None; }
    Some(HeadRef::Detached(oid.chars().take(12).collect()))
}

pub fn parse_ref_name(head: &str) -> Option<String> {
    match parse_head(head)? {
        HeadRef::Branch(branch) => Some(branch),
        HeadRef::Detached(oid) => Some(format!("detached:{oid}")),
    }
}

pub fn canonical_commit_oid(value: &str) -> Option<String> {
    let value = value.trim();
    (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| value.to_ascii_lowercase())
}

pub fn parse_gitdir_marker(marker: &str) -> Option<PathBuf> {
    let target = marker.trim().strip_prefix("gitdir:")?.trim();
    (!target.is_empty() && !target.contains('\0') && target.lines().count() == 1)
        .then(|| PathBuf::from(target))
}

fn non_empty(value: &str) -> Option<&str> { (!value.is_empty()).then_some(value) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_and_renames() {
        let summary = summarize_porcelain(" M a\n?? b\nR  old -> new\nD  gone\n");
        assert_eq!(summary, StatusSummary { staged: 2, unstaged: 1, untracked: 1, deleted: 1, total: 4 });
        assert_eq!(porcelain_paths("R  old -> new\n M a\n"), vec!["a", "new"]);
    }

    #[test]
    fn parses_branch_and_detached_head() {
        assert_eq!(parse_ref_name("ref: refs/heads/feature/x\n"), Some("feature/x".into()));
        assert_eq!(parse_ref_name("0123456789abcdef0123456789abcdef01234567\n"), Some("detached:0123456789ab".into()));
    }

    #[test]
    fn validates_commit_oid() {
        assert_eq!(canonical_commit_oid(&"A".repeat(40)), Some("a".repeat(40)));
        assert!(canonical_commit_oid("not-an-oid").is_none());
    }
}
