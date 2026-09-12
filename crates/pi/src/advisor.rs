//! The advisor (bd-cv653.3.3): a second model that reviews each agent turn
//! and injects notes inline — a quiet aside, a concern, or a hard blocker.
//!
//! The implementation lives in the `pi-advisor` workspace crate. This
//! compatibility module preserves the established
//! `pi::advisor::*` and `crate::advisor::*` paths while the runtime +
//! digest + verdict surface becomes reusable by other workspace crates.

#![forbid(unsafe_code)]

pub use pi_advisor::*;
