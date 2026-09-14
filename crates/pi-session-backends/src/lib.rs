//! Phase-2 module for `@earendil-works/pi-session-backends`:
//! session persistence backends (SQLite sidecar, JSONL tree, picker, store-v2).
//!
//! Round 18 moved the session / compaction files into `pi-coding-agent`
//! to break the `pi-coding-agent ↔ pi-session-backends` cyclic dependency.
//!
//! Round 26 begins re-housing leaves here, one at a time, to restore the
//! upstream surface.
//!
//! - `compaction_snap.rs` — Snapcompact mode (Round 26.1)
//! - `tests/session_persistence.rs` — Round-trip JSONL persistence integration
//!   test (Round 26.2, integration test under `tests/` to avoid the
//!   `pi-coding-agent ↔ pi-session-backends` cycle that would be introduced
//!   by an in-crate `pub mod`)
//!
//! Remaining session store files stay in `pi-coding-agent` until their
//! storage seams are extracted; foreign-session conversion is now independent.

#![forbid(unsafe_code)]

pub mod compaction_snap;
pub mod session_import;
