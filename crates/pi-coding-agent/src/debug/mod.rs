//! Agent-facing `debug` tool: DAP debugger driving with adapter
//! auto-selection (bd-cv653.1.2).
//!
//! The inner module `debug` (file `debug/debug.rs`) is intentionally kept
//! identical to the directory name for parity with the upstream
//! `@mariozechner/pi-coding-agent` layout, but it is exposed through this
//! façade rather than `pub mod debug` to avoid a self-referential shadow
//! in this same-named directory module. Consumers that need the inner
//! surface go through `crate::debug::*` (see `adapters`, `dap`, `session`
//! below), which already re-export the public items.

pub mod adapters;
pub mod dap;
pub mod session;

#[path = "debug.rs"]
mod inner_debug;
pub use inner_debug::*;