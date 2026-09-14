//! Phase-2 aggregator mirroring `@earendil-works/pi-protocol`:
//! JSON-RPC / framing / tail-buffer primitives shared by LSP, DAP, and
//! MCP transports.
//!
//! Round 30.1 began re-housing the framing + tail + jsonrpc modules
//! here. The transport-specific surface (`JsonRpcClient`, `await_completion`,
//! `apply_env_policy`, `reader_loop`, `PendingMap`, `SharedWriter`,
//! `ServerRequestHandler`, `lock`) stays in `pi-coding-agent/src/lsp/jsonrpc.rs`
//! because it threads through `crate::tools::ProcessGuard`,
//! `pi_error::Error`, and `crate::agent_cx::AgentCx` that this leaf crate
//! intentionally avoids.

#![forbid(unsafe_code)]

pub mod framing;
pub mod jsonrpc;
pub mod mcp;
pub mod tail;
pub mod tool_effects;
