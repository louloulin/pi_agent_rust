//! BPE token counting (bd-cv653.7.1).
//!
//! ## Stage 2 extraction
//!
//! The token-count layer (`TokenTable`, `TokenCounter`, `BpeCounter`,
//! `HeuristicCounter`, `active_counter`, `count_tokens`, `count_all_tables`,
//! `table_for_provider`) moved to the `pi-token-count` leaf crate. This
//! module re-exports every public item so existing call sites
//! (`use crate::token_count::*`, `crate::token_count::TokenTable::O200k`)
//! keep working unchanged. The inline test module (`#[cfg(test)] mod tests`)
//! lives in the leaf crate now. The `bpe-tokens` feature on the legacy
//! `pi` crate is forwarded to `pi-token-count` so `BpeCounter` stays
//! feature-gated in lock-step.

#![forbid(unsafe_code)]

pub use pi_token_count::{
    active_counter, count_all_tables, count_tokens, table_for_provider, HeuristicCounter,
    TokenCounter, TokenTable,
};

#[cfg(feature = "bpe-tokens")]
pub use pi_token_count::BpeCounter;
