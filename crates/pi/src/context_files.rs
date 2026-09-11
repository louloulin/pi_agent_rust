//! Foreign editor and agent context-rule discovery.
//!
//! The implementation lives in the `pi-context-files` workspace crate. This
//! compatibility module preserves the established `pi::context_files::*` and
//! `crate::context_files::*` paths while the context import boundary becomes
//! reusable by other workspace crates.

#![forbid(unsafe_code)]

pub use pi_context_files::*;
