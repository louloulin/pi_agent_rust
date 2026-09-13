//! Local usage statistics over session files (`pi stats`, bd-cv653.7.7).
//!
//! ## Stage 2 extraction
//!
//! The stats layer (`StatsFilter`, `TokenTotals`, `CostTotals`, `StatsReport`,
//! `ProviderModelRow`, `DayRow`, `ToolCallRow`, `collect_session_files`,
//! `aggregate`, `render_text`, `STATS_SCHEMA`) moved to the `pi-stats` leaf
//! crate. This module re-exports every public item so existing call sites
//! (`use crate::stats::*`, `crate::stats::STATS_SCHEMA`,
//! `pi::stats::StatsAggregator`, etc.) keep working unchanged. The inline
//! test module (`#[cfg(test)] mod tests`) lives in the leaf crate now.

#![forbid(unsafe_code)]

pub use pi_stats::{
    aggregate, collect_session_files, render_markdown, render_text, CostTotals, DayRow,
    ProviderModelRow, StatsFilter, StatsReport, TokenTotals, ToolCallRow, STATS_SCHEMA,
};
