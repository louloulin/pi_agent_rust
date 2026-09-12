//! Verified in-place self-updater for Pi (bd-cv653.7.10).
//!
//! The implementation lives in the `pi-self-update` workspace crate. This
//! compatibility module preserves the established
//! `pi::self_update::*` and `crate::self_update::*` paths while the
//! package-manager detection, SHA-256 verification, multi-lane artifact
//! resolution, and atomic-swap pipeline becomes reusable by other workspace
//! crates (for example, future tooling that wants to audit the current
//! binary or stage patches).
//!
//! The HTTP transport is also abstracted: `pi-self-update` ships a small
//! [`HttpFetcher`] trait so this crate does not have to depend on the
//! in-`pi` HTTP client. The production adapter [`ClientHttpFetcher`]
//! bridges that trait to `crate::http::client::Client`.

#![forbid(unsafe_code)]

pub use pi_self_update::{
    ChecksumMap, HttpFetcher, PackageManager, PlatformInfo, SelfUpdateOptions, SelfUpdateStatus,
};

// `SelfUpdater` is generic over `HttpFetcher` in the workspace crate; the
// `pi` shim publishes a concrete `SelfUpdater` that owns a
// `ClientHttpFetcher` so callers (today: `main.rs`) keep using the original
// `SelfUpdater::new()` no-arg constructor.
pub use ClientHttpFetcher as ProductionHttpFetcher;

use std::path::{Path, PathBuf};

use crate::http::client::Client;

/// Production HTTP fetcher that owns an in-`pi` `http::client::Client` and
/// adapts it to the `pi-self-update` `HttpFetcher` trait. Lets `main.rs`
/// keep its existing wiring (call `SelfUpdater::new()`, no parameters) while
/// the workspace crate stays independent of the in-`pi` HTTP stack.
#[derive(Debug, Clone)]
pub struct ClientHttpFetcher {
    client: Client,
}

impl Default for ClientHttpFetcher {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl HttpFetcher for ClientHttpFetcher {
    async fn fetch_text<'a>(
        &'a self,
        url: &'a str,
        headers: &'a [(&'a str, &'a str)],
    ) -> pi_error::Result<(u16, String)> {
        let mut req = self.client.get(url);
        for (key, value) in headers {
            req = req.header(*key, *value);
        }
        let response = req.send().await?;
        let status = response.status();
        let body = response.text().await?;
        Ok((status, body))
    }

    async fn fetch_bytes_limited<'a>(
        &'a self,
        url: &'a str,
        headers: &'a [(&'a str, &'a str)],
        max_bytes: usize,
    ) -> pi_error::Result<(u16, Vec<u8>)> {
        let mut req = self.client.get(url);
        for (key, value) in headers {
            req = req.header(*key, *value);
        }
        let response = req.send().await?;
        let status = response.status();
        let body = response.bytes_limited(max_bytes).await?;
        Ok((status, body))
    }
}

/// Concrete `SelfUpdater` bound to the in-`pi` HTTP client. Mirrors the
/// original `pi::self_update::SelfUpdater` API so the CLI wiring in
/// `main.rs` continues to compile unchanged.
pub struct SelfUpdater {
    inner: pi_self_update::SelfUpdater<ClientHttpFetcher>,
}

impl Default for SelfUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfUpdater {
    /// Construct a new updater that uses the in-`pi` HTTP client.
    pub fn new() -> Self {
        Self {
            inner: pi_self_update::SelfUpdater::new(ClientHttpFetcher::default()),
        }
    }

    /// Execute the complete self-update workflow.
    pub async fn run(&self, options: &SelfUpdateOptions) -> pi_error::Result<SelfUpdateStatus> {
        self.inner.run(options).await
    }

    /// Fetch latest release tag name from GitHub.
    pub async fn fetch_latest_version(
        &self,
        manifest_url: Option<&str>,
    ) -> pi_error::Result<String> {
        self.inner.fetch_latest_version(manifest_url).await
    }

    /// Fetch `SHA256SUMS` from the release assets.
    pub async fn fetch_checksums(
        &self,
        version: &str,
        custom_base: Option<&str>,
    ) -> pi_error::Result<ChecksumMap> {
        self.inner.fetch_checksums(version, custom_base).await
    }

    /// Perform atomic binary swap on the current executable.
    pub fn perform_atomic_swap(
        exe_path: &Path,
        new_binary_bytes: &[u8],
    ) -> pi_error::Result<PathBuf> {
        pi_self_update::SelfUpdater::<ClientHttpFetcher>::perform_atomic_swap(
            exe_path,
            new_binary_bytes,
        )
    }
}
