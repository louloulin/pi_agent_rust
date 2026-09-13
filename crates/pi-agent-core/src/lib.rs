//! Phase-2 aggregator mirroring `@earendil-works/pi-agent-core`: the
//! general-purpose agent loop, transport abstraction, and state
//! management. Downstream monorepos can depend on a single
//! `pi-agent-core` crate to reach all the underlying pieces.

#![forbid(unsafe_code)]

pub use pi_agent_cx::*;
pub use pi_agent_hub::*;
pub use pi_flake_classifier::*;
pub use pi_scheduler::*;
