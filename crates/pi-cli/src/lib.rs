//! Stage 2 leaf crate: clap derive for the `pi` binary argument surface.
//!
//! This crate owns the entire clap argument tree that used to live at
//! `crates/pi/src/cli.rs`. It is re-exported by the legacy meta-crate
//! `pi` via `pub use pi_cli::*` so every existing call site continues to
//! work without changes. No behavioral difference; this is purely a
//! workspace-level split.

#![forbid(unsafe_code)]

mod cli;

pub use cli::*;