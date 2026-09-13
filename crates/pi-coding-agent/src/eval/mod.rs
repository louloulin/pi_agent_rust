//! Eval tool (bd-cv653.1.4): persistent code kernels with Jupyter-like cell
//! semantics — state persists across cells within a session.
//!
//! See `eval.rs` (loaded here as `inner_eval`) for the public surface.
//! `js_kernel` is the secondary kernel implementation.

pub mod js_kernel;

#[path = "eval.rs"]
mod inner_eval;
pub use inner_eval::*;