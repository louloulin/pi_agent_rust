//! Phase-2 aggregator mirroring `@earendil-works/pi-ai`: the AI / Provider
//! surface (model, provider, token-count, bpe, failover, stream-rules,
//! delight, magic-keywords, dialects, embedded-assets, etc.). After
//! Round 17, every Phase-1 leaf that previously re-exported from this
//! aggregator has been inlined directly here.

#![forbid(unsafe_code)]

pub mod bpe;
pub mod delight;
pub mod dialects;
pub mod embedded_assets;
pub mod error_hints;
pub mod failover;
pub mod magic_keywords;
pub mod model;
pub mod model_routing;
pub mod model_selector;
pub mod models;
pub mod provider;
pub mod provider_metadata;
pub mod stream_rules;
pub mod token_count;
pub mod usage;
pub mod providers;