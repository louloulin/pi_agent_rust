//! Compatibility re-export for the extracted conformance matrix core.
//!
//! Matrix construction and assertion logic lives in `pi-conformance-matrix-core`.
//! The coding-agent crate keeps this module path for existing runners and users.

pub use pi_conformance_matrix_core::extension_conformance_matrix::*;
