//! Provider usage/quota readers and the `/usage` surface (bd-cv653.7.4).
//!
//! The implementation lives in the `pi-usage` workspace crate. This
//! compatibility module preserves the established
//! `pi::usage::*` and `crate::usage::*` paths while the reader
//! surface becomes reusable by other workspace crates.
//!
//! Two thin adapters live here so `pi-usage` does not have to depend on
//! the in-`pi` `AuthStorage` or `http::client::Client`:
//! - [`AuthStorageCredentials`] wraps `AuthStorage` for the
//!   [`pi_usage::CredentialLookup`] trait.
//! - [`ClientHttpFetch`] wraps `http::client::Client` for the
//!   [`pi_usage::HttpFetch`] trait.

#![forbid(unsafe_code)]

pub use pi_usage::*;

use std::time::Duration;

use crate::auth::AuthStorage;
use crate::http::client::Client;

impl CredentialLookup for AuthStorage {
    fn resolve_api_key(&self, provider: &str) -> Option<String> {
        AuthStorage::resolve_api_key(self, provider, None)
    }
}

/// Production HTTP fetcher that adapts the in-`pi` `http::client::Client`
/// to the `pi-usage` `HttpFetch` trait. Lets callers keep their existing
/// wiring (construct a `Client`, pass it in) without taking on a
/// dependency on `pi-usage` for that purpose.
///
/// The supplied `timeout` is ignored at this layer: the caller
/// (`gather_usage`) wraps each fetch in `asupersync::time::timeout`, so a
/// slow endpoint still gets budgeted even when the underlying HTTP client
/// itself runs without a per-request deadline.
pub struct ClientHttpFetch<'a>(pub &'a Client);

impl<'a> HttpFetch for ClientHttpFetch<'a> {
    async fn get_text(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        _timeout: Duration,
    ) -> pi_error::Result<(u16, String)> {
        let mut request = self.0.get(url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        Ok((status, text))
    }
}
