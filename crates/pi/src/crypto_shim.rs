//! Node.js `crypto` shim — Rust hostcalls for the QuickJS extension runtime.
//!
//! ## Stage 2 extraction
//!
//! The crypto shim surface (`NODE_CRYPTO_JS` JS source constant and
//! `register_crypto_hostcalls` registration helper) moved to the
//! `pi-crypto-shim` leaf crate. The leaf owns the rquickjs / ring /
//! sha2 / pbkdf2 / scrypt / uuid dependency cluster. This module
//! re-exports the public items so the two call sites in
//! `crates/pi/src/extensions_js.rs` (`NODE_CRYPTO_JS` literal inclusion
//! and `register_crypto_hostcalls(&global)`) keep working unchanged.

#![forbid(unsafe_code)]

pub use pi_crypto_shim::{register_crypto_hostcalls, NODE_CRYPTO_JS};
