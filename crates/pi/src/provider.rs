//! Provider protocol types and traits.
//!
//! The implementation lives in the `pi-provider` workspace crate. This
//! compatibility module preserves the established `pi::provider::*` and
//! `crate::provider::*` paths while the provider boundary becomes reusable by
//! other workspace crates.

#![forbid(unsafe_code)]

pub use pi_provider::*;
