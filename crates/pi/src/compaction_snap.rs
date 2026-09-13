//! Snapcompact compaction mode (bd-cv653.7.6).
//!
//! The implementation lives in the `pi-compaction-snap` workspace crate.
//! This compatibility module preserves the established
//! `pi::compaction_snap::*` and `crate::compaction_snap::*` paths while
//! the rasterized PNG frame encoder / detail payload surface becomes
//! reusable by other workspace crates (e.g. an offline renderer for
//! history export or session-replay tooling).

#![forbid(unsafe_code)]

pub use pi_compaction_snap::*;
