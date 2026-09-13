//! Verified in-place self-updater for Pi (bd-cv653.7.10).
//!
//! Provides `pi self-update [--version <tag>] [--check]` with:
//! - Package manager detection (Homebrew, APT, Pacman, Nix, Cargo) with refusal & remediation
//! - SHA-256 checksum verification against `SHA256SUMS` (fail-closed)
//! - Multi-lane artifact resolution (DSR bare-binary naming and release archives)
//! - Atomic swap with rollback on failed post-update smoke test
//! - Idempotent no-op when already on the target version
//!
//! The HTTP transport is abstracted through [`HttpFetcher`] so this crate
//! does not have to depend on the in-`pi` HTTP client. The `pi` crate
//! provides the production implementation for its `http::client::Client`;
//! the bundled tests use a tiny raw-TCP mock server.

use std::collections::HashMap;
use std::env;
use std::fs::{self, File, Permissions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use pi_error::{Error, Result};
use crate::version::{CURRENT_VERSION, is_newer};
use sha2::{Digest, Sha256};

const RELEASES_API_BASE: &str =
    "https://api.github.com/repos/Dicklesworthstone/pi_agent_rust/releases";
const RELEASES_DOWNLOAD_BASE: &str =
    "https://github.com/Dicklesworthstone/pi_agent_rust/releases/download";

const USER_AGENT: &str = "pi-agent-rust-self-updater";
const ACCEPT_GITHUB_JSON: &str = "application/vnd.github.v3+json";
const MAX_DOWNLOAD_BYTES: usize = 64 * 1024 * 1024;

/// Minimal HTTP abstraction used to fetch release manifests, checksums,
/// and binary artifacts.
///
/// `pi-self-update` deliberately avoids depending on the in-`pi` HTTP
/// client; the `pi` crate provides an implementation that delegates to
/// `http::client::Client` so production wiring keeps the existing
/// capability-gated transport.
pub trait HttpFetcher {
    /// Issue a GET request with the given headers and return `(status, body)`
    /// for text-style responses (JSON manifests, SHA256SUMS, etc.).
    fn fetch_text<'a>(
        &'a self,
        url: &'a str,
        headers: &'a [(&'a str, &'a str)],
    ) -> impl std::future::Future<Output = Result<(u16, String)>> + Send + 'a;

    /// Issue a GET request with the given headers and return `(status, body)`
    /// for binary responses, capped at `max_bytes`. Callers fail-closed when
    /// the response exceeds the cap (so a malicious release can't exhaust
    /// memory).
    fn fetch_bytes_limited<'a>(
        &'a self,
        url: &'a str,
        headers: &'a [(&'a str, &'a str)],
        max_bytes: usize,
    ) -> impl std::future::Future<Output = Result<(u16, Vec<u8>)>> + Send + 'a;
}

/// Lowercase hex encoder (mirrors `pi::package_manager::hex_encode`).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Known package managers that might manage the `pi` binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Homebrew,
    Apt,
    Pacman,
    Nix,
    Cargo,
    Manual,
}

impl PackageManager {
    /// Detect if the binary at `exe_path` appears to be managed by a package manager.
    pub fn detect(exe_path: &Path) -> Self {
        let path_str = exe_path.to_string_lossy();
        if path_str.contains("/Cellar/")
            || path_str.contains("/opt/homebrew/")
            || path_str.contains("/usr/local/Cellar/")
        {
            Self::Homebrew
        } else if path_str.contains("/nix/store/") {
            Self::Nix
        } else if path_str.contains("/.cargo/bin/") {
            Self::Cargo
        } else if (path_str.starts_with("/usr/bin/") || path_str.starts_with("/bin/"))
            && Path::new("/var/lib/dpkg/info").is_dir()
            && std::process::Command::new("dpkg")
                .args(["-S", &path_str])
                .output()
                .is_ok_and(|out| out.status.success())
        {
            // A binary in /usr/bin is APT-managed only when dpkg actually
            // owns it; a manual `sudo cp` install there must stay Manual or
            // the suggested `apt install --only-upgrade` can never work.
            Self::Apt
        } else {
            Self::Manual
        }
    }

    /// Suggested upgrade command if managed externally.
    pub const fn upgrade_command(&self) -> Option<&'static str> {
        match self {
            Self::Homebrew => Some("brew upgrade pi"),
            Self::Apt => Some("sudo apt update && sudo apt install --only-upgrade pi-agent-rust"),
            Self::Pacman => Some("sudo pacman -Syu pi-agent-rust"),
            Self::Nix => Some("nix-channel --update && nix-env -u pi"),
            Self::Cargo => {
                Some("cargo install --git https://github.com/Dicklesworthstone/pi_agent_rust pi")
            }
            Self::Manual => None,
        }
    }
}

/// Information about the current platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformInfo {
    pub os: &'static str,
    pub arch: &'static str,
    pub asset_platform: &'static str,
    pub target_triple: &'static str,
    pub exe_ext: &'static str,
}

impl PlatformInfo {
    /// Detect the current runtime platform.
    pub fn current() -> Option<Self> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let (asset_platform, target_triple, exe_ext) = match (os, arch) {
            ("macos", "aarch64") => ("darwin-arm64", "aarch64-apple-darwin", ""),
            ("macos", "x86_64") => ("darwin-amd64", "x86_64-apple-darwin", ""),
            ("linux", "x86_64") => ("linux-amd64", "x86_64-unknown-linux-gnu", ""),
            ("linux", "aarch64") => ("linux-arm64", "aarch64-unknown-linux-gnu", ""),
            ("windows", "x86_64") => ("windows-amd64", "x86_64-pc-windows-msvc", ".exe"),
            _ => return None,
        };

        Some(Self {
            os,
            arch,
            asset_platform,
            target_triple,
            exe_ext,
        })
    }

    /// Generate candidate asset filenames in order of preference.
    pub fn candidate_asset_names(&self, version: &str) -> Vec<String> {
        let mut candidates = Vec::new();

        // 1. DSR bare-binary naming (e.g. pi_darwin_arm64, pi_linux_amd64)
        let dsr_platform = self.asset_platform.replace('-', "_");
        candidates.push(format!("pi_{dsr_platform}{}", self.exe_ext));

        // 2. Bare binary name
        candidates.push(format!("pi{}", self.exe_ext));

        // 3. Target triple naming (e.g. pi-v0.1.0-aarch64-apple-darwin)
        candidates.push(format!(
            "pi-{version}-{}{}",
            self.target_triple, self.exe_ext
        ));
        candidates.push(format!("pi-{}{}", self.target_triple, self.exe_ext));

        // Archives (pi-<platform>.tar.xz/.zip) are deliberately NOT
        // candidates: nothing here extracts them, so an archive "install"
        // could only produce a broken binary and a rollback. Releases must
        // carry raw per-platform binaries under the names above (the
        // release pipeline uploads them alongside the archives).

        candidates
    }
}

/// Checksum map parsed from `SHA256SUMS`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChecksumMap {
    pub entries: HashMap<String, String>,
}

fn parse_checksum_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let mut parts = line.split_whitespace();
    if let (Some(hash), Some(filename)) = (parts.next(), parts.next()) {
        let clean_filename = filename.trim_start_matches('*');
        Some((clean_filename.to_string(), hash.to_lowercase()))
    } else {
        None
    }
}

impl ChecksumMap {
    /// Parse standard SHA256SUMS file content.
    pub fn parse(content: &str) -> Self {
        let entries = content.lines().filter_map(parse_checksum_line).collect();
        Self { entries }
    }

    /// Look up expected checksum for a candidate asset.
    pub fn get_hash(&self, asset_name: &str) -> Option<&str> {
        self.entries.get(asset_name).map(String::as_str)
    }

    /// Verify a byte slice against expected hash.
    pub fn verify_bytes(&self, asset_name: &str, bytes: &[u8]) -> Result<()> {
        let expected = self.get_hash(asset_name).ok_or_else(|| {
            Error::Validation(format!(
                "No checksum found for {asset_name} in SHA256SUMS (fail-closed)"
            ))
        })?;

        let actual_hash = hex_encode(&Sha256::digest(bytes)).to_lowercase();
        if actual_hash != expected {
            return Err(Error::Validation(format!(
                "Checksum mismatch for {asset_name}: expected {expected}, got {actual_hash} (fail-closed)"
            )));
        }

        Ok(())
    }
}

/// Options configuring a self-update operation.
#[derive(Debug, Clone, Default)]
pub struct SelfUpdateOptions {
    pub version: Option<String>,
    pub check: bool,
    pub custom_manifest_url: Option<String>,
    pub custom_download_base: Option<String>,
}

/// Result of a self-update execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelfUpdateStatus {
    AlreadyUpToDate {
        current_version: String,
    },
    CheckResult {
        current_version: String,
        latest_version: String,
        is_newer: bool,
        manager: PackageManager,
    },
    ManagedExternally {
        manager: PackageManager,
        upgrade_command: String,
    },
    Updated {
        previous_version: String,
        new_version: String,
        backup_path: PathBuf,
    },
}

/// In-place self-updater engine.
pub struct SelfUpdater<F: HttpFetcher> {
    client: F,
}

impl<F: HttpFetcher> SelfUpdater<F> {
    /// Construct a new updater bound to the given HTTP fetcher.
    pub fn new(client: F) -> Self {
        Self { client }
    }

    /// Fetch latest release tag name from GitHub.
    pub async fn fetch_latest_version(&self, manifest_url: Option<&str>) -> Result<String> {
        let url = manifest_url.unwrap_or(RELEASES_API_BASE);
        let api_url = if url.ends_with("/latest") || url.contains("/releases/") {
            url.to_string()
        } else {
            format!("{url}/latest")
        };

        let headers = [("User-Agent", USER_AGENT), ("Accept", ACCEPT_GITHUB_JSON)];
        let (status, body) = self.client.fetch_text(&api_url, &headers).await?;

        if !(200..300).contains(&status) {
            return Err(Error::Validation(format!(
                "Release manifest request failed with status: {status}"
            )));
        }

        let val: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| Error::Validation(format!("Invalid release JSON response: {e}")))?;

        let tag = val
            .get("tag_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("tag_name missing in release response".to_string()))?;

        Ok(tag.trim_start_matches('v').to_string())
    }

    /// Fetch `SHA256SUMS` from the release assets.
    pub async fn fetch_checksums(
        &self,
        version: &str,
        custom_base: Option<&str>,
    ) -> Result<ChecksumMap> {
        let base = custom_base.unwrap_or(RELEASES_DOWNLOAD_BASE);
        let tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{version}")
        };
        let sums_url = format!("{base}/{tag}/SHA256SUMS");

        let headers = [("User-Agent", USER_AGENT)];
        let (status, body) = self.client.fetch_text(&sums_url, &headers).await?;

        if !(200..300).contains(&status) {
            return Err(Error::Validation(format!(
                "SHA256SUMS download failed with HTTP status {status}"
            )));
        }

        Ok(ChecksumMap::parse(&body))
    }

    async fn try_download_candidate(
        &self,
        base: &str,
        tag: &str,
        candidate: &str,
        checksums: &ChecksumMap,
    ) -> Result<Option<Vec<u8>>> {
        let url = format!("{base}/{tag}/{candidate}");
        let headers = [("User-Agent", USER_AGENT)];

        let (status, bytes) = match self
            .client
            .fetch_bytes_limited(&url, &headers, MAX_DOWNLOAD_BYTES)
            .await
        {
            Ok(pair) => pair,
            Err(_) => return Ok(None),
        };

        if !(200..300).contains(&status) {
            return Ok(None);
        }

        checksums.verify_bytes(candidate, &bytes)?;
        Ok(Some(bytes))
    }

    /// Download and verify binary artifact bytes.
    pub async fn download_and_verify(
        &self,
        platform: &PlatformInfo,
        version: &str,
        checksums: &ChecksumMap,
        custom_base: Option<&str>,
    ) -> Result<(String, Vec<u8>)> {
        let base = custom_base.unwrap_or(RELEASES_DOWNLOAD_BASE);
        let tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{version}")
        };

        let candidates = platform.candidate_asset_names(version);

        for candidate in candidates {
            if let Some(bytes) = self
                .try_download_candidate(base, &tag, &candidate, checksums)
                .await?
            {
                return Ok((candidate, bytes));
            }
        }

        Err(Error::Validation(format!(
            "No compatible binary candidate found for platform {} in release {tag}",
            platform.asset_platform
        )))
    }

    /// Execute the complete self-update workflow.
    pub async fn run(&self, options: &SelfUpdateOptions) -> Result<SelfUpdateStatus> {
        let current_exe = env::current_exe().map_err(|e| {
            Error::Io(Box::new(std::io::Error::other(format!(
                "Failed to locate current executable path: {e}"
            ))))
        })?;

        // Package manager check
        let manager = PackageManager::detect(&current_exe);
        if manager != PackageManager::Manual
            && !options.check
            && let Some(cmd) = manager.upgrade_command()
        {
            return Ok(SelfUpdateStatus::ManagedExternally {
                manager,
                upgrade_command: cmd.to_string(),
            });
        }

        let target_version = match &options.version {
            Some(v) => v.trim_start_matches('v').to_string(),
            None => {
                self.fetch_latest_version(options.custom_manifest_url.as_deref())
                    .await?
            }
        };

        let current_ver = CURRENT_VERSION.trim_start_matches('v');

        if options.check {
            return Ok(SelfUpdateStatus::CheckResult {
                current_version: current_ver.to_string(),
                latest_version: target_version.clone(),
                is_newer: is_newer(current_ver, &target_version),
                manager,
            });
        }

        if current_ver == target_version {
            return Ok(SelfUpdateStatus::AlreadyUpToDate {
                current_version: current_ver.to_string(),
            });
        }

        let platform = PlatformInfo::current().ok_or_else(|| {
            Error::Validation(format!(
                "Unsupported operating system or architecture: {} {}",
                env::consts::OS,
                env::consts::ARCH
            ))
        })?;

        let checksums = self
            .fetch_checksums(&target_version, options.custom_download_base.as_deref())
            .await?;

        let (_asset_name, bytes) = self
            .download_and_verify(
                &platform,
                &target_version,
                &checksums,
                options.custom_download_base.as_deref(),
            )
            .await?;

        let backup = SelfUpdater::<F>::perform_atomic_swap(&current_exe, &bytes)?;

        Ok(SelfUpdateStatus::Updated {
            previous_version: current_ver.to_string(),
            new_version: target_version,
            backup_path: backup,
        })
    }

    /// Perform atomic binary swap on the current executable.
    pub fn perform_atomic_swap(exe_path: &Path, new_binary_bytes: &[u8]) -> Result<PathBuf> {
        let parent_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
        let pid = std::process::id();
        let tmp_path = parent_dir.join(format!(".pi-update-tmp.{pid}"));
        let backup_path = parent_dir.join(format!(".pi-update-backup.{pid}"));

        // 1. Write new binary to tmp file
        {
            let mut tmp_file = File::create(&tmp_path).map_err(|e| {
                Error::Io(Box::new(std::io::Error::other(format!(
                    "Failed to create temporary update file: {e}"
                ))))
            })?;
            tmp_file.write_all(new_binary_bytes).map_err(|e| {
                Error::Io(Box::new(std::io::Error::other(format!(
                    "Failed to write update bytes: {e}"
                ))))
            })?;
            tmp_file.flush().map_err(|e| {
                Error::Io(Box::new(std::io::Error::other(format!(
                    "Failed to flush update file: {e}"
                ))))
            })?;
        }

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = Permissions::from_mode(0o755);
            let _ = fs::set_permissions(&tmp_path, perms);
        }

        // 2. Rename existing executable to backup
        if let Err(e) = fs::rename(exe_path, &backup_path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(Error::Io(Box::new(std::io::Error::other(format!(
                "Failed to backup existing binary {}: {e}",
                exe_path.display()
            )))));
        }

        // 3. Rename tmp to executable
        if let Err(e) = fs::rename(&tmp_path, exe_path) {
            // Restore backup
            let _ = fs::rename(&backup_path, exe_path);
            let _ = fs::remove_file(&tmp_path);
            return Err(Error::Io(Box::new(std::io::Error::other(format!(
                "Failed to install new binary {}: {e}",
                exe_path.display()
            )))));
        }

        // 4. Run smoke test
        let smoke_check = Command::new(exe_path).arg("--version").output();
        let smoke_ok = match smoke_check {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if !smoke_ok {
            // Rollback immediately
            let _ = fs::rename(&backup_path, exe_path);
            return Err(Error::Validation(
                "Post-update smoke test (--version) failed; rolled back to previous binary"
                    .to_string(),
            ));
        }

        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::thread;

    /// Mock HTTP fetcher that proxies through a hand-rolled raw TCP server.
    /// Lets the tests live in this crate without dragging in the production
    /// HTTP client stack.
    #[allow(dead_code)]
    struct MockHttp {
        url: String,
        server: Arc<std::sync::Mutex<Option<thread::JoinHandle<()>>>>,
    }

    impl MockHttp {
        fn spawn_text(status: u16, body: &str) -> Self {
            let body = body.to_string();
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind text server");
            let addr = listener.local_addr().expect("text server addr");
            let handle = thread::spawn(move || {
                let (mut stream, _) = listener.accept().expect("accept text request");
                let mut request = [0u8; 2048];
                let _ = stream.read(&mut request);
                let status_text = match status {
                    200 => "OK",
                    404 => "Not Found",
                    500 => "Internal Server Error",
                    _ => "Test Response",
                };
                let response = format!(
                    "HTTP/1.1 {status} {status_text}\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write text response");
            });
            Self {
                url: format!("http://{addr}/releases"),
                server: Arc::new(std::sync::Mutex::new(Some(handle))),
            }
        }

        #[allow(dead_code)]
        fn join(&self) {
            if let Some(handle) = self.server.lock().unwrap().take() {
                handle.join().expect("join mock server");
            }
        }
    }

    /// Parses an HTTP/1.1 response from a freshly-opened socket to the
    /// recorded mock URL. Used by both text and bytes fixtures.
    fn fetch_mock(url: &str) -> Result<(u16, Vec<u8>)> {
        let prefix = url
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("");
        let mut stream = std::net::TcpStream::connect(prefix)
            .map_err(|err| Error::api(format!("mock connect: {err}")))?;
        let path = url
            .split_once('/')
            .map(|(_, rest)| format!("/{rest}"))
            .unwrap_or_default();
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nUser-Agent: pi-agent-rust-self-updater\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|err| Error::api(format!("mock write: {err}")))?;
        let mut raw = Vec::new();
        std::io::Read::read_to_end(&mut stream, &mut raw)
            .map_err(|err| Error::api(format!("mock read: {err}")))?;
        let split = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or_else(|| Error::api("mock: missing header terminator".to_string()))?;
        let header_bytes = &raw[..split];
        let body = raw[split + 4..].to_vec();
        let header_text = std::str::from_utf8(header_bytes)
            .map_err(|err| Error::api(format!("mock: bad utf8: {err}")))?;
        let status_line = header_text
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

    impl HttpFetcher for MockHttp {
        async fn fetch_text<'a>(
            &'a self,
            _url: &'a str,
            _headers: &'a [(&'a str, &'a str)],
        ) -> Result<(u16, String)> {
            let (status, body) = fetch_mock(&self.url)?;
            let text =
                String::from_utf8(body).map_err(|err| Error::api(format!("mock: utf8: {err}")))?;
            Ok((status, text))
        }

        async fn fetch_bytes_limited<'a>(
            &'a self,
            _url: &'a str,
            _headers: &'a [(&'a str, &'a str)],
            _max_bytes: usize,
        ) -> Result<(u16, Vec<u8>)> {
            fetch_mock(&self.url)
        }
    }

    #[test]
    fn checksum_map_parser_and_verifier() {
        let sample_sums = r"
# SHA256SUMS for v0.2.0
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  pi-v0.2.0-x86_64-unknown-linux-gnu
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *pi_darwin_arm64
";
        let map = ChecksumMap::parse(sample_sums);
        assert_eq!(
            map.get_hash("pi-v0.2.0-x86_64-unknown-linux-gnu"),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
        assert_eq!(
            map.get_hash("pi_darwin_arm64"),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );

        // Verify bytes for "abc" -> ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        assert!(map.verify_bytes("pi_darwin_arm64", b"abc").is_ok());
        assert!(map.verify_bytes("pi_darwin_arm64", b"corrupted").is_err());
        assert!(map.verify_bytes("unknown_asset", b"abc").is_err());
    }

    #[test]
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    fn platform_detection_and_candidates() {
        if let Some(plat) = PlatformInfo::current() {
            let candidates = plat.candidate_asset_names("0.2.0");
            assert!(!candidates.is_empty());
            assert!(candidates.iter().any(|c| c.contains("pi")));
            // No extractor exists: archive candidates would only ever
            // produce a failed install + rollback.
            assert!(
                candidates.iter().all(|c| {
                    let lower = c.to_ascii_lowercase();
                    !lower.ends_with(".tar.gz")
                        && !lower.ends_with(".tar.xz")
                        && !lower.ends_with(".zip")
                }),
                "{candidates:?}"
            );
        }
    }

    #[test]
    fn package_manager_detection() {
        assert_eq!(
            PackageManager::detect(Path::new("/opt/homebrew/bin/pi")),
            PackageManager::Homebrew
        );
        assert_eq!(
            PackageManager::detect(Path::new("/usr/local/Cellar/pi/0.1.0/bin/pi")),
            PackageManager::Homebrew
        );
        assert_eq!(
            PackageManager::detect(Path::new("/nix/store/xyz-pi/bin/pi")),
            PackageManager::Nix
        );
        assert_eq!(
            PackageManager::detect(Path::new("/home/user/.cargo/bin/pi")),
            PackageManager::Cargo
        );
        assert_eq!(
            PackageManager::detect(Path::new("/home/user/.local/bin/pi")),
            PackageManager::Manual
        );
    }

    #[test]
    fn hex_encode_basic() {
        assert_eq!(hex_encode(&[]), "");
        assert_eq!(hex_encode(&[0x00, 0xff, 0xab, 0x12]), "00ffab12");
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn fetch_latest_version_parses_tag_from_release_manifest() {
        asupersync::test_utils::run_test(|| async {
            let body = r#"{"tag_name":"v9.9.9","name":"Release 9.9.9"}"#;
            let mock = MockHttp::spawn_text(200, body);
            let updater = SelfUpdater::new(mock);

            let tag = updater.fetch_latest_version(None).await.expect("tag");
            assert_eq!(tag, "9.9.9");
        });
    }

    #[test]
    fn fetch_latest_version_rejects_non_2xx() {
        asupersync::test_utils::run_test(|| async {
            let mock = MockHttp::spawn_text(404, "not found");
            let updater = SelfUpdater::new(mock);

            let err = updater.fetch_latest_version(None).await.unwrap_err();
            assert!(
                matches!(err, Error::Validation(_)),
                "expected Validation, got {err:?}"
            );
        });
    }

    #[test]
    fn fetch_latest_version_fails_on_missing_tag_name() {
        asupersync::test_utils::run_test(|| async {
            let mock = MockHttp::spawn_text(200, r#"{"name":"missing tag"}"#);
            let updater = SelfUpdater::new(mock);

            let err = updater.fetch_latest_version(None).await.unwrap_err();
            assert!(
                matches!(err, Error::Validation(_)),
                "expected Validation, got {err:?}"
            );
        });
    }

    #[test]
    fn fetch_checksums_parses_sha256_sums_file() {
        asupersync::test_utils::run_test(|| async {
            let sums = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  pi_linux_amd64\nba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *pi_darwin_arm64\n";
            let mock = MockHttp::spawn_text(200, sums);
            let updater = SelfUpdater::new(mock);

            let map = updater
                .fetch_checksums("0.2.0", None)
                .await
                .expect("checksums");
            assert_eq!(map.entries.len(), 2);
            assert_eq!(
                map.get_hash("pi_darwin_arm64"),
                Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
            );
        });
    }

    #[test]
    fn fetch_checksums_rejects_non_2xx() {
        asupersync::test_utils::run_test(|| async {
            let mock = MockHttp::spawn_text(500, "boom");
            let updater = SelfUpdater::new(mock);

            let err = updater.fetch_checksums("0.2.0", None).await.unwrap_err();
            assert!(matches!(err, Error::Validation(_)));
        });
    }

    #[test]
    fn download_and_verify_returns_first_matching_candidate() {
        asupersync::test_utils::run_test(|| async {
            // Body bytes are SHA256("abc") = ba7816bf...
            let body_bytes = b"abc".to_vec();
            let mock =
                MockHttp::spawn_text(200, &String::from_utf8(body_bytes).expect("ascii body"));
            let updater = SelfUpdater::new(mock);

            let sums = ChecksumMap::parse(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *pi_darwin_arm64\n",
            );
            // Use a fabricated platform so we hit the bare "pi" candidate name.
            let platform = PlatformInfo {
                os: "macos",
                arch: "aarch64",
                asset_platform: "darwin-arm64",
                target_triple: "aarch64-apple-darwin",
                exe_ext: "",
            };

            let (asset, bytes) = updater
                .download_and_verify(&platform, "0.2.0", &sums, None)
                .await
                .expect("download_and_verify");
            assert!(!asset.is_empty());
            assert_eq!(bytes, b"abc");
        });
    }
}
