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
//! - `extension_popularity.rs` — Popularity signal snapshotting for
//!   extension candidates; fetches GitHub + npm metrics (Round 29.2)
//! - `extension_validation.rs` — Validation evidence + dry-run probes
//!   for extension candidates (Round 29.2)
//! - `skills_managed.rs` — User-managed skill CRUD (create / update /
//!   delete / list) backed by a per-user ledger under the agent global dir
//!   (Round 30.4)
//! - `platform.rs` — OS / filesystem abstraction: effective-mode access
//!   context, trusted symlink resolution, OS + arch naming (Round 31)
//! - `crash.rs` — Crash bundle capture + redacted panic hook + crash
//!   list/show/delete/emit_startup_notice for the agent recovery flow
//!   (Round 32)
//! - `version.rs` — Background version check: semver compare, GitHub
//!   release parsing, cached refresh + `HttpFetch` trait seam
//!   (Round 33)
//! - `turn_recovery.rs` — Classify model `StopReason` into
//!   `RecoveryClass` (retry / continue / give-up) for the agent loop
//!   (Round 34)
//! - `status_line.rs` — Powerline status line builder: presets,
//!   separators, segment IDs, responsive dropping, accent-hue hash
//!   (Round 35)
//! - `secrets.rs` — credential detection, outbound gating, and the
//!   session-scoped placeholder vault (Round 37)
//! - `undo.rs` — bounded content-addressed file mutation history with
//!   undo/redo and external-change protection (Round 38)
//! - `commit_split.rs` — dependency-ordered atomic git commit planning and
//!   execution with secret-safe messages (Round 39)
//! - `model_routing.rs` — provider health and cost routing evidence over a
//!   minimal `RoutingModel` trait seam (Round 40)

#![forbid(unsafe_code)]

pub mod buffer_shim;
pub mod commit_split;
pub mod crash;
pub mod extension_inclusion;
pub mod extension_license;
pub mod extension_popularity;
pub mod extension_validation;
pub mod file_lock;
pub mod hostcall_egraph;
pub mod hostcall_io_uring_lane;
pub mod hostcall_queue;
pub mod hostcall_rewrite;
pub mod hostcall_s3_fifo;
pub mod hostcall_superinstructions;
pub mod hostcall_trace_jit;
pub mod model_routing;
pub mod platform;
pub mod secrets;
pub mod skills_managed;
pub mod status_line;
pub mod turn_recovery;
pub mod tool_policy;
pub mod undo;
pub mod version;
pub mod hostcall_amac;
