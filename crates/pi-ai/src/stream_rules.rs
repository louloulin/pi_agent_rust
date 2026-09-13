//! Time-Traveling Stream Rules (TTSR) engine and Grievances Ledger (bd-cv653.3.4).
//!
//! The implementation lives in the `pi-stream-rules` workspace crate. This
//! compatibility module preserves the established
//! `pi::stream_rules::*` and `crate::stream_rules::*` paths while the TTSR
//! engine, rule store, and grievances ledger become reusable by other
//! workspace crates.

#![forbid(unsafe_code)]

pub use pi_stream_rules::*;
