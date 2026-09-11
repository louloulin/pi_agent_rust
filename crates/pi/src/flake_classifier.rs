// Conformance flake classifier (bd-k5q5.5.4)
//
// Classifies test failures as deterministic or transient based on
// known flake patterns.  Used by CI retry logic and triage tooling.
//
// ## Stage 2 extraction
//
// The classifier surface (`FlakeCategory`, `FlakeClassification`,
// `FlakeEvent`, `classify_failure`, `RetryPolicy`) moved to the
// `pi-flake-classifier` leaf crate. This module re-exports every
// public item so existing call sites (`use crate::flake_classifier::*`)
// keep working unchanged. The inline test module (`#[cfg(test)] mod tests`)
// lives in the leaf crate now.

#![forbid(unsafe_code)]

pub use pi_flake_classifier::{
    classify_failure, FlakeCategory, FlakeClassification, FlakeEvent, RetryPolicy,
};
