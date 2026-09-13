//! Phase-2 aggregator placeholder mirroring `@earendil-works/pi-agent-core`:
//! agent runtime core primitives. After Round 18 all of the files that
//! previously lived here were absorbed into `pi-coding-agent` to break
//! the `pi-coding-agent ↔ pi-agent-core` cycle. This crate remains so
//! the 11-package surface mirrors the upstream repository structure;
//! downstream consumers should depend on `pi-coding-agent` instead.

#![forbid(unsafe_code)]

pub mod flake_classifier;
pub mod scheduler;