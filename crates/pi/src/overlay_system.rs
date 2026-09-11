//! Unified overlay and set-piece surfaces (legacy re-export shim).
//!
//! ## Stage 2 extraction
//!
//! Overlay surfaces relocated to the `pi-overlay-system` leaf crate.
//! This module re-exports every public item from the leaf so the two
//! call sites in `interactive.rs` and `interactive/view.rs`
//! (`crate::overlay_system::WelcomeScreen::default()`) keep working
//! unchanged.

#![forbid(unsafe_code)]

pub use pi_overlay_system::{
    OverlayEntry, OverlayKind, OverlayStack, ToastLevel, ToastNotification, ToastQueue,
    WelcomeScreen,
};
