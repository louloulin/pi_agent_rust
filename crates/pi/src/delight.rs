//! Premium delight layer (OMP-ADOPT / bd-cv653.9.9).
//!
//! ## Stage 2 extraction
//!
//! The delight layer (`ShimmerMode`, `compute_shimmer_intensity`, `render_sparkline`,
//! `Particle`, `FireworksState`, `format_terminal_title`) moved to the
//! `pi-delight` leaf crate. This module re-exports every public item so existing
//! call sites (`use crate::delight::*`, `crate::delight::format_terminal_title`)
//! keep working unchanged. The inline test module (`#[cfg(test)] mod tests`)
//! lives in the leaf crate now.

#![forbid(unsafe_code)]

pub use pi_delight::{
    compute_shimmer_intensity, format_terminal_title, render_sparkline, FireworksState, Particle,
    ShimmerMode,
};
