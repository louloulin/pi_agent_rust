//! Platform identity and filesystem-access policy utilities.
//!
//! The implementation lives in the `pi-platform` workspace crate. This
//! compatibility module preserves the established `pi::platform::*` and
//! `crate::platform::*` paths while the platform boundary becomes reusable by
//! other workspace crates.

#![forbid(unsafe_code)]

pub use pi_platform::*;
