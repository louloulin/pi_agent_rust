//! Provider-independent LLM wire contracts and policies.
//!
//! This crate intentionally contains no HTTP, authentication, provider SDK, or
//! runtime dependencies.  It is the stable seam shared by transports and the
//! coding-agent provider implementations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Generic request envelope passed to a provider transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestEnvelope {
    pub model: String,
    pub input: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Generic response envelope retained for provider-neutral adapters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub output: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Normalized classification used by retry and failover callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    RateLimited,
    Overloaded,
    Server,
    Timeout,
    Network,
    Authentication,
    InvalidRequest,
    Other,
}

impl FailureKind {
    pub const fn retryable(self) -> bool {
        matches!(self, Self::RateLimited | Self::Overloaded | Self::Server | Self::Timeout | Self::Network)
    }
}

/// Transport-independent provider failure details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub kind: FailureKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
}

/// Bounded exponential retry policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self { Self { max_attempts: 3, base_delay_ms: 1_000, max_delay_ms: 60_000 } }
}

impl RetryPolicy {
    /// Return the delay for a zero-based retry index, honoring server hints.
    pub fn delay_ms(self, retry_index: u32, server_hint_ms: Option<u64>) -> u64 {
        let exponential = self.base_delay_ms.saturating_mul(2u64.saturating_pow(retry_index.min(63)));
        server_hint_ms.unwrap_or(exponential).min(self.max_delay_ms)
    }
}

/// Parse the standard `Retry-After` value (seconds or milliseconds).
pub fn parse_retry_after(value: &str) -> Option<u64> {
    let value = value.trim();
    value.parse::<u64>().ok().map(|seconds| seconds.saturating_mul(1_000))
        .or_else(|| value.strip_suffix("ms")?.trim().parse().ok())
}

/// Classify an HTTP response without exposing transport details.
pub fn classify_status(status: u16) -> FailureKind {
    match status {
        401 | 403 => FailureKind::Authentication,
        400..=499 if status != 408 && status != 429 => FailureKind::InvalidRequest,
        408 => FailureKind::Timeout,
        429 => FailureKind::RateLimited,
        500..=599 if status == 503 || status == 529 => FailureKind::Overloaded,
        500..=599 => FailureKind::Server,
        _ => FailureKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn retry_after_supports_seconds_and_ms() { assert_eq!(parse_retry_after("2"), Some(2_000)); assert_eq!(parse_retry_after("250ms"), Some(250)); }
    #[test] fn retry_is_bounded() { assert_eq!(RetryPolicy { max_delay_ms: 2_500, ..Default::default() }.delay_ms(4, None), 2_500); }
    #[test] fn status_categories_are_stable() { assert_eq!(classify_status(429), FailureKind::RateLimited); assert!(classify_status(503).retryable()); assert!(!classify_status(400).retryable()); }
}
