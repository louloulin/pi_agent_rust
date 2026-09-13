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
//! - `hostcall_superinstructions.rs` — Trace-driven superinstruction compiler
//!   and execution runtime (Round 27.3)
//! - `hostcall_io_uring_lane.rs` — io_uring dispatch lane policy and
//!   fallback telemetry (Round 27.4)
//! - `hostcall_s3_fifo.rs` — S3-FIFO eviction policy with tier telemetry
//!   (Round 27.5)
//! - `hostcall_trace_jit.rs` — Trace-JIT compiler producing guarded
//!   superinstruction plans (Round 27.6)
//! - `hostcall_queue.rs` — Hostcall dispatch queue with overflow handling
//!   and S3-FIFO eviction telemetry (Round 27.7)
//! - `hostcall_egraph.rs` — Equality-saturation e-graph for discovering
//!   hostcall rewrite plans (Round 27.8)
//! - `extension_license.rs` — SPDX license detection and screening for Pi
//!   extension candidates (Round 29.1)
//! - `extension_inclusion.rs` — Final inclusion list generation: merges
//!   scoring tiers, license verdicts, validation evidence into a pinned
//!   inclusion list (Round 29.1)

#![forbid(unsafe_code)]

pub mod buffer_shim;
pub mod extension_inclusion;
pub mod extension_license;
pub mod file_lock;
pub mod hostcall_egraph;
pub mod hostcall_io_uring_lane;
pub mod hostcall_queue;
pub mod hostcall_rewrite;
pub mod hostcall_s3_fifo;
pub mod hostcall_superinstructions;
pub mod hostcall_trace_jit;
