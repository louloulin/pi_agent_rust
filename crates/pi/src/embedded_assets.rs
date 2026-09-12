//! Losslessly compressed text resources bundled into the shipping binary.
//!
//! The implementation lives in the `pi-embedded-assets` workspace crate.
//! This compatibility module preserves the established
//! `pi::embedded_assets::*` and `crate::embedded_assets::*` paths while the
//! resource boundary becomes reusable by other workspace crates.

#![forbid(unsafe_code)]

pub use pi_embedded_assets::*;
