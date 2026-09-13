//! Final inclusion list generation for Pi extension candidates.
//!
//! The implementation lives in the `pi-extension-inclusion` workspace
//! crate. This compatibility module preserves the established
//! `pi::extension_inclusion::*` and `crate::extension_inclusion::*` paths
//! while the inclusion list, version pin, and normalized manifest hash
//! surface becomes reusable by other workspace crates (e.g. the
//! extension scoring / conformance matrix pipelines).

#![forbid(unsafe_code)]

pub use pi_extension_inclusion::*;
