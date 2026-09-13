//! Phase-2 module for `@earendil-works/pi-session-backends`:
//! session persistence backends (SQLite sidecar, JSONL tree, picker, store-v2).
//!
//! Round 18 moved the session / compaction files into `pi-coding-agent`
//! to break the `pi-coding-agent ↔ pi-session-backends` cyclic dependency.
//!
//! Round 26 begins re-housing them here, one at a time, to restore the
//! upstream surface. `compaction_snap.rs` (Snapcompact mode) is the first
//! piece to migrate because it has only `pi_ai`, `base64`, and `serde`
//! dependencies and five intra-workspace call sites.

#![forbid(unsafe_code)]

pub mod compaction_snap;
