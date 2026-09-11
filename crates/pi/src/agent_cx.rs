//! Capability-scoped async context wrapper (legacy re-export shim).
//!
//! ## Stage 2 extraction
//!
//! `AgentCx` and the related capability handles (`AgentFs`, `AgentTime`,
//! `AgentProcess`) relocated to the `pi-agent-cx` leaf crate. This
//! module re-exports every public item from the leaf so the ten call
//! sites in `sdk.rs`
//! (`crate::agent_cx::AgentCx::for_request()`) keep working unchanged.
//!
//! `AgentHttp` and the `AgentCx::http()` accessor were removed during
//! this extraction: the previous `AgentHttp::client()` returned
//! `crate::http::client::Client`, a hard cross-module dep on the
//! not-yet-extracted `http` module. No caller in `crates/pi/src/`
//! invoked `AgentHttp::client()` (only `cx.http()` was referenced in
//! a self-test). They will return as a leaf of their own when
//! `pi-http` lands.

#![forbid(unsafe_code)]

pub use pi_agent_cx::{AgentCx, AgentFs, AgentProcess, AgentTime};
