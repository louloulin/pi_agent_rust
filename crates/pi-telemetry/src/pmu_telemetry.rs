//! PMU-guided stall-cycle elimination and microarchitectural regression budgets.
//!
//! ## Stage 2 extraction
//!
//! The PMU telemetry primitives moved to the `pi-pmu-telemetry` leaf crate.
//! This module re-exports every public item so existing call sites
//! (`use crate::pmu_telemetry::*`, `crate::pmu_telemetry::PmuSample`,
//! `pi::pmu_telemetry::PMU_TELEMETRY_SCHEMA`, etc.) keep working unchanged.

#![forbid(unsafe_code)]

pub use pi_pmu_telemetry::{
    PmuBudgetVerdict, PmuOpportunityRanker, PmuOptimizationOpportunity, PmuRegressionBudget,
    PmuSample, PMU_TELEMETRY_SCHEMA,
};
