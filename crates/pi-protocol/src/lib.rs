//! Phase-2 aggregator mirroring `@earendil-works/pi-protocol`:
//! transport-neutral CBOR protocol plumbing. The Rust implementation
//! uses JSON-RPC + SSE today; the leaf crates (`pi-jsonrpc` / `pi-sse`)
//! are the low-level building blocks.

#![forbid(unsafe_code)]

pub use pi_jsonrpc::*;
pub use pi_sse::*;
