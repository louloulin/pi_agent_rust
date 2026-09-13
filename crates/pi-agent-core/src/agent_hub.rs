//! Agent hub registry (bd-cv653.5.3): a session-scoped roster of spawned
//! subagent children with transcript persistence, steering delivery, kill,
//! revive, and a minimal peer-messaging bus.
//!
//! The implementation lives in the `pi-agent-hub` workspace crate. This
//! compatibility module preserves the established
//! `pi::agent_hub::*` and `crate::agent_hub::*` paths while the roster
//! becomes reusable by other workspace crates (for example, future tooling
//! that wants to audit session-local subagent history).

#![forbid(unsafe_code)]

pub use pi_agent_hub::*;
