//! Phase-2 module for `@earendil-works/pi-tui`:
//! terminal image rendering for the TUI extension surface.
//!
//! Round 28 begins re-housing leaves here, one at a time, to restore the
//! upstream surface.
//!
//! - `terminal_images.rs` — terminal-specific image placeholder rendering
//!   for the `placeholder(mime_type, ...)` API (Round 28.1)
//! - `tui.rs` — rich_rust-backed console + TUI-aware log redirection
//!   (Round 28.2)
//! - `autocomplete.rs` — file/slash/template/skill autocomplete provider,
//!   rendering-agnostic (Round 28.3)
//! - `file_refs.rs` — file URL / quoted-ref parsing helpers shared between
//!   the bubbletea and ftui editor stacks (Round 28.4)
//! - `text_utils.rs` — display-width truncation helpers for the message
//!   renderer (Round 28.5)
//! - `overlay_system.rs` — unified overlay stack + set-piece surfaces
//!   (Esc-stack, toast notifications, welcome / help / picker modals)
//!   (Round 30.2)
//! - `gallery.rs` — visual component gallery harness: every tool card
//!   and UI component in every lifecycle state for visual QA / regression
//!   gating (Round 30.3)

#![forbid(unsafe_code)]

pub mod autocomplete;
pub mod file_refs;
pub mod gallery;
pub mod overlay_system;
pub mod terminal_images;
pub mod text_utils;
# tui.rs was split into the dependency-light pi-tui-core crate in Round 67;
# keep the legacy module path as a compatibility re-export.
// Round 67: the implementation lives in pi-tui-core; retain the historical
// `pi_tui::tui` path for callers while avoiding duplicate rendering code.
pub use pi_tui_core::tui;
