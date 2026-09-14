//! Pure glob and ignore-pattern primitives shared by Pi search tools.

use std::path::{Path, PathBuf};

pub use globset::{Glob, GlobMatcher};
pub use ignore::{gitignore, overrides};

/// Bound used when expanding a git `core.excludesFile` path.
pub const GLOBAL_IGNORE_PATH_MAX_BYTES: usize = 2 * 1024 * 1024;

/// Parse gitconfig's `core.excludesFile`, preserving the ignore crate's
/// historical broad `~` replacement while bounding allocation growth.
pub fn parse_gitconfig_excludes_path(
    data: &[u8],
    home_dir: Option<&Path>,
) -> std::io::Result<Option<PathBuf>> {
    let re = regex::bytes::Regex::new(
        r#"(?im-u)^\s*excludesfile\s*=\s*"?\s*(\S+?)\s*"?\s*$"#,
    )
    .expect("valid git excludesFile regex");
    let Some(candidate) = re
        .captures(data)
        .and_then(|captures| captures.get(1))
        .and_then(|capture| std::str::from_utf8(capture.as_bytes()).ok())
    else {
        return Ok(None);
    };
    let tilde_count = candidate.bytes().filter(|byte| *byte == b'~').count();
    let expanded_len = home_dir.map_or(Some(candidate.len()), |home| {
        candidate
            .len()
            .checked_sub(tilde_count)?
            .checked_add(tilde_count.checked_mul(home.to_string_lossy().len())?)
    });
    if expanded_len.is_none_or(|len| len > GLOBAL_IGNORE_PATH_MAX_BYTES) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("expanded git global ignore path exceeds {GLOBAL_IGNORE_PATH_MAX_BYTES} bytes"),
        ));
    }
    Ok(Some(PathBuf::from(home_dir.map_or_else(
        || candidate.to_owned(),
        |home| candidate.replace('~', &home.to_string_lossy()),
    ))))
}

/// Build a glob matcher for filename-shaped patterns.
pub fn compile_filename_glob(pattern: &str) -> std::io::Result<GlobMatcher> {
    Glob::new(pattern)
        .map(|glob| glob.compile_matcher())
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_expands_excludes_file() {
        let path = parse_gitconfig_excludes_path(
            b"[core]\n excludesFile = ~/global.ignore\n",
            Some(Path::new("/home/test")),
        )
        .expect("parse")
        .expect("path");
        assert_eq!(path, PathBuf::from("/home/test/global.ignore"));
    }

    #[test]
    fn rejects_expansion_that_exceeds_bound() {
        let home = "h".repeat(128);
        let count = GLOBAL_IGNORE_PATH_MAX_BYTES / home.len() + 1;
        let error = parse_gitconfig_excludes_path(
            format!("excludesFile = {}\n", "~".repeat(count)).as_bytes(),
            Some(Path::new(&home)),
        )
        .expect_err("bounded expansion");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }
}
