//! Server-Sent Events (SSE) parser for streaming LLM responses.
//!
//! ## Stage 2 extraction
//!
//! The SSE parser (`SseEvent`, `SseParser`, `SseStream`) moved to the
//! `pi-sse` leaf crate. This module re-exports every public item so existing
//! call sites (`use crate::sse::*`, `crate::sse::SseEvent`,
//! `pi::sse::SseParser`, etc.) keep working unchanged. The inline test
//! module (`#[cfg(test)] mod tests`) lives in the leaf crate now.

#![forbid(unsafe_code)]

pub use pi_sse::{SseEvent, SseParser, SseStream};
