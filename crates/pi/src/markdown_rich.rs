//! Markdown, math, mermaid, and visual excellence (OMP-ADOPT / bd-cv653.9.7).
//!
//! ## Stage 2 extraction
//!
//! The markdown-enrichment surface (`HighlightLanguage`,
//! `latex_to_unicode`, `render_hex_swatches`, `enrich_markdown`,
//! `format_osc8_link`, `render_mermaid_diagram`) moved to the
//! `pi-markdown-rich` leaf crate. This module re-exports every public
//! item so existing call sites (`use crate::markdown_rich::*`,
//! `crate::markdown_rich::enrich_markdown`) keep working unchanged.
//! The inline test module (`#[cfg(test)] mod tests`) lives in the leaf
//! crate now.

#![forbid(unsafe_code)]

pub use pi_markdown_rich::{
    enrich_markdown, format_osc8_link, latex_to_unicode, render_hex_swatches,
    render_mermaid_diagram, HighlightLanguage,
};
