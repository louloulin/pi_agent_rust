//! Multi-root workspace state and the unified path-confinement helper
//! (bd-cv653.3.12).
//!
//! The implementation lives in the `pi-workspace` workspace crate. This
//! compatibility module preserves the established
//! `pi::workspace::*` and `crate::workspace::*` paths while the workspace
//! root set and the unified confinement gate become reusable by other
//! workspace crates.

#![forbid(unsafe_code)]

pub use pi_workspace::*;
