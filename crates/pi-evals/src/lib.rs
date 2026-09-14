//! Phase-2 aggregator placeholder mirroring `@earendil-works/pi-evals`:
//! the evaluation harness. After Round 18 the files that previously
//! lived here were absorbed into `pi-coding-agent` to break the
//! `pi-coding-agent ↔ pi-evals` cycle. This crate remains so the
//! 11-package surface mirrors the upstream repository structure;
//! downstream consumers should depend on `pi-coding-agent` instead.

#![forbid(unsafe_code)]

// Empty: all eval / lsp modules were moved to `pi-coding-agent` in
// Round 18 to break the dependency cycle. This crate is retained as a
// marker so `crates/pi`'s facade can still re-export an `evals`
// namespace.