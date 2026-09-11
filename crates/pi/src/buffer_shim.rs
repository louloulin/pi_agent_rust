//! Node.js `Buffer` shim — pure-JS implementation for the QuickJS extension runtime.
//!
//! ## Stage 2 extraction
//!
//! The `NODE_BUFFER_JS` JS source constant moved to the `pi-buffer-shim`
//! leaf crate (zero external Rust deps — the payload is a JS string).
//! This module re-exports the constant so the lone call site in
//! `crates/pi/src/extensions_js.rs` (`crate::buffer_shim::NODE_BUFFER_JS`)
//! keeps working unchanged.

#![forbid(unsafe_code)]

pub use pi_buffer_shim::NODE_BUFFER_JS;
