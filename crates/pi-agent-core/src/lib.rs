//! Phase-2 aggregator mirroring `@earendil-works/pi-agent-core`: the
//! agent runtime core (cx, hub, scheduler, flake-classifier, etc.). After
//! Round 17, every Phase-1 leaf has been inlined directly here.

#![forbid(unsafe_code)]

pub mod agent;
pub mod agent_cx;
pub mod agent_hub;
pub mod flake_classifier;
pub mod handoff;
pub mod memory;
pub mod resource_governor;
pub mod scheduler;
pub mod skills_managed;
pub mod subagents;