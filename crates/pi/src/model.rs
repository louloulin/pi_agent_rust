//! Shared Pi wire-format types.
//!
//! The implementation lives in the `pi-model` workspace crate. This module
//! remains as a re-export so the established `pi::model::*` API and internal
//! `crate::model::*` paths continue to work while the workspace is split into
//! focused crates.

#![forbid(unsafe_code)]

pub use pi_model::*;
