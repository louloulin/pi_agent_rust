//! Phase-2 aggregator mirroring `@earendil-works/pi-ai`: the unified LLM
//! API surface (model + provider + tokenization + policy).
//!
//! This crate is a thin re-export of the existing Phase-1 leaf crates so
//! downstream monorepos and SDK consumers can write `use pi_ai::*;` and
//! reach every ai-related public API through a single dependency.

#![forbid(unsafe_code)]

pub use pi_bpe::*;
pub use pi_delight::*;
pub use pi_dialects::*;
pub use pi_embedded_assets::*;
pub use pi_failover::*;
pub use pi_magic_keywords::*;
pub use pi_model::*;
pub use pi_provider::*;
pub use pi_provider_metadata::*;
pub use pi_stream_rules::*;
pub use pi_token_count::*;
