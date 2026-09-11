//! Per-model tool-call dialect converters (bd-cv653.7.8).
//!
//! ## Stage 2 extraction
//!
//! The dialect layer (`Dialect`, `dialect_for_model`, `RepairCandidate`,
//! `RepairEntry`, `RepairLedger`, `extract_text_tool_calls`,
//! `strip_candidates`) moved to the `pi-dialects` leaf crate. This module
//! re-exports every public item so existing call sites
//! (`use crate::dialects::*`, `crate::dialects::Dialect`,
//! `pi::dialects::RepairLedger`, etc.) keep working unchanged. The inline
//! test module (`#[cfg(test)] mod tests`) lives in the leaf crate now.

#![forbid(unsafe_code)]

pub use pi_dialects::{
    dialect_for_model, extract_text_tool_calls, strip_candidates, Dialect, RepairCandidate,
    RepairEntry, RepairLedger,
};
