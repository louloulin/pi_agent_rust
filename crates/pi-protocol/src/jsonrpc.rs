//! JSON-RPC 2.0 framing primitives shared by LSP, DAP, and MCP transports.
//!
//! This crate owns the protocol-layer building blocks that used to live at
//! `crates/pi/src/lsp/jsonrpc.rs`:
//!
//! - [`RpcErrorObject`] — the wire-level error object
//! - [`TransportError`] — typed failure taxonomy (Server / Closed / Io)
//! - [`ServerNotification`] — server-pushed method+params
//! - [`EnvPolicy`] / [`MCP_ENV_ALLOWLIST`] — server-process environment composition
//! - [`encode_frame`] / [`read_frame`] / [`read_frame_with_scratch`] —
//!   `Content-Length`-framed I/O (no LSP/MCP-specific state)
//! - [`PublicTailBuffer`] — bounded ring buffer for stderr tail capture
//! - [`CompletionWaitError`] — wait outcome enum (the actual
//!   `await_completion` helper stays in the legacy crate because it threads
//!   through `AgentCx` and `asupersync::time`)
//!
//! Crate-public items mirror the visibility they had inside `pi`'s
//! `lsp::jsonrpc` module: `pub` for anything that crossed module boundaries,
//! `pub(crate)` would normally hide the I/O helpers, but as a leaf crate they
//! are exposed at `pub` for any consumer that needs them.
//!
//! No behavioral change vs. the legacy in-line module; this is a pure
//! workspace-level extraction.

#![forbid(unsafe_code)]

mod framing;
mod tail;

pub use framing::{
    EnvPolicy, MCP_ENV_ALLOWLIST, RpcErrorObject, ServerNotification, TransportError, encode_frame,
    find_subslice, parse_content_length, read_frame, read_frame_with_scratch,
};
pub use tail::{PublicTailBuffer, TailBuffer};

/// Why a completion wait ended without a value.
///
/// The actual polling loop (`await_completion`) lives in the legacy
/// meta-crate because it threads through `AgentCx` + `asupersync::time`;
/// only the outcome enum is shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionWaitError {
    /// Deadline exceeded.
    Timeout,
    /// Ambient cancellation fired.
    Cancelled,
    /// The sender dropped without sending.
    Closed,
}
