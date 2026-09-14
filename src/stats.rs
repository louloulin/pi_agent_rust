//! Compatibility facade for the statistics core.
//!
//! The implementation lives in `pi-stats-core`; this module keeps the
//! existing `pi::stats` API used by the CLI while leaving runtime/UI code in
//! the main package.

pub use pi_stats_core::*;
