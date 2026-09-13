//! `pi-mono` is the workspace aggregator crate for `pi.rs` Phase-1.
//!
//! It re-exports the public surface of the primary `pi` crate so consumers
//! that previously depended on the monolithic `pi_agent_rust` package can
//! migrate incrementally:
//!
//! ```ignore
//! use pi_mono::pi;
//! ```
//!
//! Stage-1 keeps this aggregator thin; subsequent phases of the
//! `pi-rs-modularization-phase1` plan will add re-exports for chord / tui /
//! telemetry / ai / agent / protocol / client / server / coding-agent / evals
//! mirroring the upstream `@earendil-works/*` package layout.

#![forbid(unsafe_code)]

pub use pi;
