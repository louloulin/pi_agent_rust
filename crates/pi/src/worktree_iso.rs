//! Workspace isolation for subagents (bd-cv653.5.2).
//!
//! The implementation lives in the `pi-worktree-iso` workspace crate.
//! This compatibility module preserves the established
//! `pi::worktree_iso::*` and `crate::worktree_iso::*` paths while the
//! isolation lifecycle becomes reusable by other workspace crates.

#![forbid(unsafe_code)]

pub use pi_worktree_iso::*;
