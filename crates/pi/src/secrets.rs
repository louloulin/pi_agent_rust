//! Credential-shape detection and session-scoped secrets vault.
//!
//! The implementation lives in the `pi-secrets` workspace crate. This
//! compatibility module preserves the established `pi::secrets::*` and
//! `crate::secrets::*` paths while the secrets boundary becomes reusable by
//! other workspace crates.

#![forbid(unsafe_code)]

pub use pi_secrets::*;
