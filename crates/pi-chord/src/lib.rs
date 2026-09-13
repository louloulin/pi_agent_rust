//! Phase-2 module for `@earendil-works/pi-chord`:
//! hostcall / buffer / file-lock runtime.
//!
//! Round 18 moved the chord leaves into `pi-coding-agent` to break the
//! `pi-coding-agent ↔ pi-chord` cycle. Round 27 begins re-housing them
//! here, one at a time, to restore the upstream surface.
//!
//! - `buffer_shim.rs` — Node `Buffer` polyfill source for the QuickJS extension
//!   runtime (Round 27.1)
//! - `file_lock.rs` — proper-lockfile-compatible directory lock for shared
//!   settings/auth/session-index files (Round 27.1)
//! - `hostcall_rewrite.rs` — Constrained hostcall rewrite planner for hot-path
//!   marshalling (Round 27.2)

#![forbid(unsafe_code)]

pub mod buffer_shim;
pub mod file_lock;
pub mod hostcall_rewrite;
