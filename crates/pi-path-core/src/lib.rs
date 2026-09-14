//! Small, dependency-light path primitives shared by Pi crates.
//!
//! These functions are lexical (they do not touch the filesystem), preserve
//! platform prefixes/root directories, and safely handle non-UTF-8 path
//! components by operating on `OsStr`/`OsString`.

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

/// Remove `.` and lexical `..` segments without allowing an absolute path to
/// escape its root. Relative leading `..` segments are retained.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    let mut parts: Vec<OsString> = Vec::new();
    let mut rooted = false;
    let mut prefixed = false;
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => { out.push(prefix.as_os_str()); prefixed = true; }
            Component::RootDir => { out.push(component.as_os_str()); rooted = true; }
            Component::CurDir => {}
            Component::ParentDir => match parts.last() {
                Some(last) if last != OsStr::new("..") => { parts.pop(); }
                _ if !rooted && !prefixed => parts.push(OsString::from("..")),
                _ => {}
            },
            Component::Normal(part) => parts.push(part.to_os_string()),
        }
    }
    for part in parts { out.push(part); }
    out
}

/// Resolve a path against `cwd`, expanding the current user's home directory.
pub fn resolve(input: &str, cwd: &Path) -> PathBuf {
    let trimmed = input.trim();
    let base = dirs::home_dir().unwrap_or_else(|| cwd.to_path_buf());
    let path = if trimmed == "~" {
        base
    } else if let Some(rest) = trimmed.strip_prefix("~/") {
        base.join(rest)
    } else if let Some(rest) = trimmed.strip_prefix('~') {
        base.join(rest)
    } else {
        let candidate = Path::new(trimmed);
        if candidate.is_absolute() { candidate.to_path_buf() } else { cwd.join(candidate) }
    };
    normalize(&path)
}

/// Resolve a path against a caller-provided base directory without touching IO.
pub fn resolve_from(input: &str, base_dir: &Path) -> PathBuf {
    let trimmed = input.trim();
    if trimmed == "~" { return dirs::home_dir().unwrap_or_else(|| base_dir.to_path_buf()); }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        return dirs::home_dir().unwrap_or_else(|| base_dir.to_path_buf()).join(rest);
    }
    let path = Path::new(trimmed);
    if path.is_absolute() { path.to_path_buf() } else { base_dir.join(path) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_absolute_and_relative_paths() {
        assert_eq!(normalize(Path::new("/a/./b/../c")), PathBuf::from("/a/c"));
        assert_eq!(normalize(Path::new("a/../../b")), PathBuf::from("../b"));
    }
    #[test]
    fn preserves_non_utf8_components() {
        #[cfg(unix)] { use std::os::unix::ffi::OsStringExt; let p = PathBuf::from(OsString::from_vec(vec![b'a', 0xff])); assert_eq!(normalize(&p), p); }
    }
    #[test]
    fn resolves_relative_paths() { assert_eq!(resolve("src/../Cargo.toml", Path::new("/tmp/pi")), PathBuf::from("/tmp/pi/Cargo.toml")); }
}
