//! Phase-2 aggregator placeholder mirroring `@earendil-works/pi-session-backends`:
//! session persistence backends (SQLite sidecar, JSONL tree, picker, store-v2).
//! After Round 18 the session / compaction files that previously lived here
//! were moved into `pi-coding-agent` to break the `pi-coding-agent ↔
//! pi-session-backends` cyclic dependency. This crate remains so the
//! 11-package surface mirrors the upstream repository structure;
//! downstream consumers should depend on `pi-coding-agent` instead.

#![forbid(unsafe_code)]

// Empty: all session / compaction / sqlite modules were moved to
// `pi-coding-agent` in Round 18 to break the dependency cycle. This crate
// is retained as a marker so `crates/pi`'s facade can still re-export a
// `session_backends` namespace.