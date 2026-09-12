//! Background version check — queries GitHub releases for newer versions.
//!
//! Checks are non-blocking, cached for 24 hours, and configurable via
//! `check_for_updates` in settings.json.
//!
//! The actual HTTP call is abstracted through [`HttpFetch`] so this crate
//! does not have to depend on the in-`pi` HTTP client. The `pi` crate
//! provides the production implementation for its `http::client::Client`;
//! the bundled tests use a tiny raw-TCP mock server.

use std::path::{Path, PathBuf};
use std::time::Duration;

use pi_error::{Error, Result};
use semver::{BuildMetadata, Version};

/// Current crate version (from Cargo.toml).
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long to cache the version check result (24 hours).
const CACHE_TTL_SECS: u64 = 24 * 60 * 60;
pub(crate) const RELEASE_CHECK_TIMEOUT: Duration = Duration::from_secs(10);
const RELEASES_URL: &str =
    "https://api.github.com/repos/Dicklesworthstone/pi_agent_rust/releases/latest";

/// Minimal HTTP abstraction used to fetch the latest release JSON.
///
/// `pi-version` deliberately avoids depending on the in-`pi` HTTP client;
/// the `pi` crate provides an implementation that delegates to
/// `http::client::Client` so production wiring keeps the existing
/// capability-gated transport.
pub trait HttpFetch {
    /// Issue a GET request and return `(status, body)`.
    ///
    /// # Errors
    /// Surfaces transport-level errors as `Error::api` so callers can
    /// collapse any failure into `VersionCheckResult::Failed`.
    fn fetch_release<'a>(
        &'a self,
        url: &'a str,
        timeout: Duration,
    ) -> impl std::future::Future<Output = Result<(u16, String)>> + Send + 'a;
}

/// Result of a version check.
#[derive(Debug, Clone)]
pub enum VersionCheckResult {
    /// A newer version is available.
    UpdateAvailable { latest: String },
    /// Already on the latest (or newer) version.
    UpToDate,
    /// Check failed (network error, parse error, etc.) — fail silently.
    Failed,
}

/// Compare two semver-like version strings (e.g. "0.1.0" vs "0.2.0").
///
/// Returns `true` if `latest` is strictly newer than `current`.
#[must_use]
pub fn is_newer(current: &str, latest: &str) -> bool {
    match (parse_semver_like(current), parse_semver_like(latest)) {
        (Some(current), Some(latest)) => latest > current,
        _ => false,
    }
}

fn parse_semver_like(version: &str) -> Option<Version> {
    let version = version.strip_prefix('v').unwrap_or(version).trim();
    if version.is_empty() {
        return None;
    }
    if let Ok(parsed) = Version::parse(version) {
        return Some(strip_build_metadata(parsed));
    }

    let suffix_idx = version.find(['-', '+']).unwrap_or(version.len());
    let (core, suffix) = version.split_at(suffix_idx);
    if core.split('.').count() != 2 {
        return None;
    }

    Version::parse(&format!("{core}.0{suffix}"))
        .ok()
        .map(strip_build_metadata)
}

fn strip_build_metadata(mut version: Version) -> Version {
    version.build = BuildMetadata::EMPTY;
    version
}

/// Path to the version check cache file.
fn cache_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("pi").join(".version_check_cache")
}

/// Read a cached version if the cache is fresh (within TTL).
#[must_use]
pub fn read_cached_version() -> Option<String> {
    read_cached_version_at(&cache_path())
}

fn read_cached_version_at(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = modified.elapsed().ok()?;
    if age.as_secs() > CACHE_TTL_SECS {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let version = content.trim().to_string();
    if version.is_empty() {
        return None;
    }
    Some(version)
}

/// Write a version to the cache file.
pub fn write_cached_version(version: &str) {
    write_cached_version_at(&cache_path(), version);
}

fn write_cached_version_at(path: &Path, version: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, version);
}

/// Refresh the cached latest-release version when the cache is missing, stale,
/// or malformed. Returns the resulting status after the refresh attempt.
pub async fn refresh_cache_if_stale<F: HttpFetch>(client: &F) -> VersionCheckResult {
    refresh_cache_if_stale_at(&cache_path(), client, CURRENT_VERSION, RELEASES_URL).await
}

async fn refresh_cache_if_stale_at<F: HttpFetch>(
    path: &Path,
    client: &F,
    current_version: &str,
    release_url: &str,
) -> VersionCheckResult {
    if let Some(latest) = read_cached_version_at(path)
        && parse_semver_like(&latest).is_some()
    {
        return version_status_for(current_version, latest);
    }

    match fetch_latest_release_version_from_url(client, release_url).await {
        Ok(latest) => {
            write_cached_version_at(path, &latest);
            version_status_for(current_version, latest)
        }
        Err(err) => {
            tracing::debug!(error = %err, "background version check failed");
            VersionCheckResult::Failed
        }
    }
}

fn version_status_for(current_version: &str, latest: String) -> VersionCheckResult {
    if is_newer(current_version, &latest) {
        VersionCheckResult::UpdateAvailable { latest }
    } else {
        VersionCheckResult::UpToDate
    }
}

async fn fetch_latest_release_version_from_url<F: HttpFetch>(
    client: &F,
    release_url: &str,
) -> Result<String> {
    let (status, body) = client.fetch_release(release_url, RELEASE_CHECK_TIMEOUT).await?;
    if status != 200 {
        return Err(Error::api(format!(
            "GitHub release lookup failed with status {status}"
        )));
    }

    parse_github_release_version(&body)
        .ok_or_else(|| Error::api("GitHub release lookup response missing tag_name".to_string()))
}

/// Check the latest version from cache or return None if cache is stale/missing.
///
/// The actual HTTP check is performed separately (by the caller spawning
/// a background task with the HTTP client).
#[must_use]
pub fn check_cached() -> VersionCheckResult {
    check_cached_at(&cache_path(), CURRENT_VERSION)
}

fn check_cached_at(path: &Path, current_version: &str) -> VersionCheckResult {
    let Some(latest) = read_cached_version_at(path) else {
        return VersionCheckResult::Failed;
    };

    match (
        parse_semver_like(current_version),
        parse_semver_like(&latest),
    ) {
        (Some(current), Some(latest_version)) => {
            if latest_version > current {
                VersionCheckResult::UpdateAvailable { latest }
            } else {
                VersionCheckResult::UpToDate
            }
        }
        _ => VersionCheckResult::Failed,
    }
}

/// Parse the latest version from a GitHub releases API JSON response.
///
/// Expects the response from `https://api.github.com/repos/OWNER/REPO/releases/latest`.
#[must_use]
pub fn parse_github_release_version(json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let tag = value.get("tag_name")?.as_str()?;
    // Strip leading 'v' if present
    let version = tag.strip_prefix('v').unwrap_or(tag);
    if version.trim().is_empty() {
        return None;
    }
    Some(version.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::thread;

    /// Mock HTTP fetcher that proxies through a hand-rolled raw TCP server.
    /// Lets the tests live in this crate without dragging in the production
    /// HTTP client stack.
    struct MockHttp {
        url: String,
        server: Arc<std::sync::Mutex<Option<thread::JoinHandle<()>>>>,
    }

    impl MockHttp {
        fn spawn(status: u16, body: &'static str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind release server");
            let addr = listener.local_addr().expect("release server addr");
            let body = body.to_string();
            let handle = thread::spawn(move || {
                let (mut stream, _) = listener.accept().expect("accept release request");
                let mut request = [0u8; 2048];
                let _ = stream.read(&mut request);
                let status_text = match status {
                    200 => "OK",
                    404 => "Not Found",
                    500 => "Internal Server Error",
                    _ => "Test Response",
                };
                let response = format!(
                    "HTTP/1.1 {status} {status_text}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write release response");
            });
            Self {
                url: format!("http://{addr}/releases/latest"),
                server: Arc::new(std::sync::Mutex::new(Some(handle))),
            }
        }

        fn join(&self) {
            if let Some(handle) = self.server.lock().unwrap().take() {
                handle.join().expect("join release server");
            }
        }

        /// Mock fetcher that never opens a connection: keeps cached tests
        /// deterministic even when the dev environment blocks sockets.
        fn disconnected() -> Self {
            Self {
                url: "http://127.0.0.1:9/releases/latest".to_string(),
                server: Arc::new(std::sync::Mutex::new(None)),
            }
        }
    }

    impl HttpFetch for MockHttp {
        async fn fetch_release(&self, _url: &str, _timeout: Duration) -> Result<(u16, String)> {
            // Parse the response we just wrote via a fresh socket.
            use std::io::Write as _;
            use std::net::TcpStream;
            let mut stream =
                TcpStream::connect(self.url.trim_start_matches("http://").split('/').next().unwrap())
                    .map_err(|err| Error::api(format!("mock connect: {err}")))?;
            let request = format!(
                "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept: application/vnd.github+json\r\nConnection: close\r\n\r\n",
                self.url.split_once('/').map(|(_, rest)| format!("/{rest}")).unwrap_or_default()
            );
            stream
                .write_all(request.as_bytes())
                .map_err(|err| Error::api(format!("mock write: {err}")))?;
            let mut raw = String::new();
            std::io::Read::read_to_string(&mut stream, &mut raw)
                .map_err(|err| Error::api(format!("mock read: {err}")))?;
            let split = raw
                .find("\r\n\r\n")
                .ok_or_else(|| Error::api("mock: missing header terminator".to_string()))?;
            let headers = &raw[..split];
            let body = raw[split + 4..].to_string();
            let status_line = headers
                .lines()
                .next()
                .ok_or_else(|| Error::api("mock: empty status line".to_string()))?;
            let status: u16 = status_line
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| Error::api("mock: bad status".to_string()))?;
            Ok((status, body))
        }
    }

    #[test]
    fn is_newer_basic() {
        assert!(is_newer("0.1.0", "0.2.0"));
        assert!(is_newer("0.1.0", "1.0.0"));
        assert!(is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn is_newer_same_version() {
        assert!(!is_newer("1.0.0", "1.0.0"));
    }

    #[test]
    fn is_newer_current_is_newer() {
        assert!(!is_newer("2.0.0", "1.0.0"));
    }

    #[test]
    fn is_newer_with_v_prefix() {
        assert!(is_newer("v0.1.0", "v0.2.0"));
        assert!(is_newer("0.1.0", "v0.2.0"));
        assert!(is_newer("v0.1.0", "0.2.0"));
    }

    #[test]
    fn is_newer_with_prerelease() {
        assert!(is_newer("1.2.3-dev", "1.2.3"));
        assert!(is_newer("1.2.3-dev", "1.3.0"));
        assert!(!is_newer("1.2.3", "1.2.3-dev"));
    }

    #[test]
    fn is_newer_ignores_build_metadata() {
        assert!(!is_newer("1.2.3+build.1", "1.2.3+build.2"));
        assert!(!is_newer("1.2.3", "1.2.3+build.2"));
    }

    #[test]
    fn is_newer_invalid_versions() {
        assert!(!is_newer("not-a-version", "1.0.0"));
        assert!(!is_newer("1.0.0", "not-a-version"));
        assert!(!is_newer("", ""));
    }

    #[test]
    fn parse_github_release_version_valid() {
        let json = r#"{"tag_name": "v0.2.0", "name": "Release 0.2.0"}"#;
        assert_eq!(
            parse_github_release_version(json),
            Some("0.2.0".to_string())
        );
    }

    #[test]
    fn parse_github_release_version_no_v_prefix() {
        let json = r#"{"tag_name": "0.2.0"}"#;
        assert_eq!(
            parse_github_release_version(json),
            Some("0.2.0".to_string())
        );
    }

    #[test]
    fn parse_github_release_version_invalid_json() {
        assert_eq!(parse_github_release_version("not json"), None);
    }

    #[test]
    fn parse_github_release_version_missing_tag() {
        let json = r#"{"name": "Release"}"#;
        assert_eq!(parse_github_release_version(json), None);
    }

    #[test]
    fn parse_github_release_version_rejects_empty_tag() {
        assert_eq!(parse_github_release_version(r#"{"tag_name": ""}"#), None);
        assert_eq!(parse_github_release_version(r#"{"tag_name": "v"}"#), None);
    }

    #[test]
    fn cache_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache");

        write_cached_version_at(&path, "1.2.3");
        assert_eq!(read_cached_version_at(&path), Some("1.2.3".to_string()));
    }

    #[test]
    fn cache_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent");
        assert_eq!(read_cached_version_at(&path), None);
    }

    #[test]
    fn cache_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache");
        std::fs::write(&path, "").unwrap();
        assert_eq!(read_cached_version_at(&path), None);
    }

    #[test]
    fn check_cached_invalid_cached_version_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache");
        write_cached_version_at(&path, "not-a-version");
        assert!(matches!(
            check_cached_at(&path, "1.2.3"),
            VersionCheckResult::Failed
        ));
    }

    #[test]
    fn refresh_cache_if_stale_fetches_and_writes_latest_release() {
        asupersync::test_utils::run_test(|| async {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("cache");
            let mock = MockHttp::spawn(200, r#"{"tag_name":"v9.9.9"}"#);

            let result = refresh_cache_if_stale_at(&path, &mock, "1.0.0", &mock.url).await;

            assert!(matches!(
                result,
                VersionCheckResult::UpdateAvailable { latest } if latest == "9.9.9"
            ));
            assert_eq!(read_cached_version_at(&path), Some("9.9.9".to_string()));
            mock.join();
        });
    }

    #[test]
    fn refresh_cache_if_stale_uses_fresh_cache_without_network() {
        asupersync::test_utils::run_test(|| async {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("cache");
            write_cached_version_at(&path, "1.2.3");

            let mock = MockHttp::disconnected();
            let result = refresh_cache_if_stale_at(
                &path,
                &mock,
                "1.0.0",
                "http://127.0.0.1:9/releases/latest",
            )
            .await;

            assert!(matches!(
                result,
                VersionCheckResult::UpdateAvailable { latest } if latest == "1.2.3"
            ));
            assert_eq!(read_cached_version_at(&path), Some("1.2.3".to_string()));
        });
    }

    #[test]
    fn refresh_cache_if_stale_replaces_malformed_cache() {
        asupersync::test_utils::run_test(|| async {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("cache");
            write_cached_version_at(&path, "definitely-not-a-version");
            let mock = MockHttp::spawn(200, r#"{"tag_name":"v2.1.0"}"#);

            let result = refresh_cache_if_stale_at(&path, &mock, "2.0.0", &mock.url).await;

            assert!(matches!(
                result,
                VersionCheckResult::UpdateAvailable { latest } if latest == "2.1.0"
            ));
            assert_eq!(read_cached_version_at(&path), Some("2.1.0".to_string()));
            mock.join();
        });
    }

    #[test]
    fn refresh_cache_if_stale_fail_closed_on_invalid_release_payload() {
        asupersync::test_utils::run_test(|| async {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("cache");
            let mock = MockHttp::spawn(200, r#"{"name":"missing tag"}"#);

            let result = refresh_cache_if_stale_at(&path, &mock, "1.0.0", &mock.url).await;

            assert!(matches!(result, VersionCheckResult::Failed));
            assert_eq!(read_cached_version_at(&path), None);
            mock.join();
        });
    }
}
