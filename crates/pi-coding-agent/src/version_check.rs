//! Background version check — queries GitHub releases for newer versions.
//!
//! The implementation lives in the `pi-version` workspace crate. This
//! compatibility module preserves the established
//! `pi::version_check::*` and `crate::version_check::*` paths while the
//! cache + semver pipeline becomes reusable by other workspace crates.

#![forbid(unsafe_code)]

pub use pi_version::*;

use std::time::Duration;

/// Production HTTP fetcher that adapts the in-`pi` `http::client::Client`
/// to the `pi-version` `HttpFetch` trait. Lets `interactive.rs` keep its
/// existing wiring (construct a `Client`, pass it in) without taking on
/// a dependency on `pi-version` for that purpose.
pub struct ClientHttpFetch<'a>(pub &'a crate::http::client::Client);

impl<'a> pi_version::HttpFetch for ClientHttpFetch<'a> {
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