//! Compatibility adapter for the `pi-chord::version` check API.

#![forbid(unsafe_code)]

pub use pi_chord::version::{
    CURRENT_VERSION, HttpFetch, VersionCheckResult, check_cached, is_newer,
    parse_github_release_version, read_cached_version, refresh_cache_if_stale,
    write_cached_version,
};

use std::time::Duration;

/// Production HTTP fetcher that adapts the in-`pi` `http::client::Client`
/// to the `pi-chord::version` `HttpFetch` trait. Lets `interactive.rs`
/// keep its existing wiring (construct a `Client`, pass it in) without
/// taking on a dependency on the upstream `http` adapter directly.
pub struct ClientHttpFetch<'a>(pub &'a crate::http::client::Client);

impl<'a> pi_chord::version::HttpFetch for ClientHttpFetch<'a> {
    async fn fetch_release(&self, url: &str, timeout: Duration) -> pi_error::Result<(u16, String)> {
        let response = self
            .0
            .get(url)
            .timeout(timeout)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        Ok((status, body))
    }
}
