//! Phase-2 aggregator mirroring `@earendil-works/pi-ai`: the AI / Provider
//! surface (model, provider, token-count, bpe, failover, stream-rules,
//! delight, magic-keywords, dialects, embedded-assets, etc.). After
//! Round 17, every Phase-1 leaf that previously re-exported from this
//! aggregator has been inlined directly here.

#![allow(unsafe_code)]
// bpe.rs is a verbatim port of the upstream o200k/cl100k tables that uses
// a private ThreadId u64 counter via transmute (rust-lang/rust#67939).
// The unsafe block is contained to a single test helper and stays narrow.

pub mod bpe;
pub use bpe::{BpeError, BpeResult};
pub mod delight;
pub mod dialects;
pub mod embedded_assets;
pub mod error_hints;
pub mod failover;
pub mod magic_keywords;
pub mod model;
// moved to pi-coding-agent: model_routing
// moved to pi-coding-agent: model_selector
// moved to pi-coding-agent in Round 18: models
pub mod provider;
pub mod provider_metadata;
pub mod sse;
pub mod stream_rules;
pub mod token_count;
// moved to pi-coding-agent in Round 18: usage
pub mod secret_screener;