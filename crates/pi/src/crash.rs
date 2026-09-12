//! Crash capture: redacted crash bundles from panics and fatal signals
//! (bd-cv653.7.12).
//!
//! The implementation lives in the `pi-crash` workspace crate. This
//! compatibility module preserves the established `pi::crash::*` and
//! `crate::crash::*` paths while the crash bundle pipeline becomes
//! reusable by other workspace crates.

#![forbid(unsafe_code)]

pub use pi_crash::*;