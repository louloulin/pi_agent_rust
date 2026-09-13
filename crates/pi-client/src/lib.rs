//! Phase-2 aggregator placeholder mirroring `@earendil-works/pi-client`:
//! client-side integrations (web_remote, web_search, xdev). After Round
//! 18 the files that previously lived here were absorbed into
//! `pi-coding-agent` to break the `pi-coding-agent ↔ pi-client` cycle.
//! This crate remains so the 11-package surface mirrors the upstream
//! repository structure; downstream consumers should depend on
//! `pi-coding-agent` instead.

#![forbid(unsafe_code)]

// Empty: all client / web_remote / web_search / xdev modules were moved
// to `pi-coding-agent` in Round 18 to break the dependency cycle. This
// crate is retained as a marker so `crates/pi`'s facade can still
// re-export a `client` namespace.