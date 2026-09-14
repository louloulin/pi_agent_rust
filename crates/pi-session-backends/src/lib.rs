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
//! The remaining session / sqlite / picker files (`session.rs`,
//! `session_index.rs`, `session_picker.rs`, `session_sqlite.rs`,
//! `session_store_v2.rs`, `session_import.rs`) stay in `pi-coding-agent`
//! for now; they will be migrated in later rounds once the inter-module
//! dependency edges are analysed and broken with `EventSource` / `StoreKind`
//! traits.

#![forbid(unsafe_code)]

pub mod compaction_snap;
