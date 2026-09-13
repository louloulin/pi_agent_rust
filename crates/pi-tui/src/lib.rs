//! Phase-2 aggregator mirroring `@earendil-works/pi-tui`: Terminal UI
//! rendering and input handling.
//!
//! Most of the TUI surface currently lives inlined inside `crates/pi`
//! (`interactive_ftui.rs`, `interactive/agent.rs`, `interactive/commands.rs`,
//! `tui.rs`, `keybindings.rs`, `theme.rs`). Future Phase-2 rounds will
//! extract a `pi-ui` core leaf crate and re-export it through this
//! aggregator.

#![forbid(unsafe_code)]
