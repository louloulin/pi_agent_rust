//! Vendored tiktoken BPE rank-table loader (legacy re-export shim).
//!
//! ## Stage 2 extraction
//!
//! The vendored BPE engine relocated to the `pi-bpe` leaf crate. This
//! module re-exports every public item from the leaf so existing call
//! sites in `token_count.rs` that import via `crate::bpe::*` keep
//! working unchanged. `BpeError` and `BpeResult` are leaf-local: no
//! caller of `CoreBPE::new` observed the concrete error type, so the
//! legacy `crate::error::{Error, Result}` surface remains the only
//! error vocabulary at the meta-crate level.

#![forbid(unsafe_code)]

pub use pi_bpe::{
    byte_pair_encode, byte_pair_split, CoreBPE, DecodeError, DecodeKeyError, EncodeError, Rank,
};
