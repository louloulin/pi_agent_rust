//! Phase-2 aggregator mirroring `@earendil-works/pi-coding-agent`: the
//! coding-agent CLI surface (tools, session, extensions, persistence,
//! platform glue). After Round 17, every Phase-1 leaf that previously
//! re-exported from this aggregator has been inlined directly here. The
//! actual `pi` binary entry is declared in `Cargo.toml` (`[[bin]] name = "pi"`)
//! and lives at `src/main.rs`.

#![forbid(unsafe_code)]

// Inlined from the Phase-1 leaves (Round 17).
pub mod advisor;
pub mod cli;
pub mod conformance;
pub mod context_files;
pub mod crash;
pub mod crypto_shim;
pub mod error;
pub mod extension_replay;
pub mod extension_scoring;
pub mod gallery;
pub mod markdown_rich;
pub mod overlay_system;
pub mod platform;
pub mod secret_screener;
pub mod secrets;
pub mod self_update;
pub mod stats;
pub mod status_line;
pub mod swarm_activity_ledger;
pub mod swarm_progress_slo;
pub mod turn_recovery;
pub mod undo;
pub mod version;
pub mod workspace;

// Files relocated from `crates/pi/src/` in Round 13.
pub mod app;
pub mod approval;
pub mod ask;
pub mod ast_tools;
pub mod auth;
pub mod bash_mediation;
pub mod browser;
pub mod btw;
pub mod checkpoint;
pub mod commit_split;
pub mod completions;
pub mod computer;
pub mod config;
pub mod conformance_shapes;
pub mod current_time;
pub mod doctor;
pub mod enforcement;
pub mod extension_conformance_matrix;
pub mod extension_dispatcher;
pub mod extension_events;
pub mod extension_inclusion;
pub mod extension_index;
pub mod extension_license;
pub mod extension_popularity;
pub mod extension_preflight;
pub mod extension_tools;
pub mod extension_validation;
pub mod extensions_api;
pub mod extensions_js;
pub mod gc;
pub mod hub;
pub mod keybindings;
pub mod mcp;
pub mod media_tools;
pub mod perf_build;
pub mod permissions;
pub mod pi_wasm;
pub mod resources;
pub mod security_scan;
pub mod semantic_workspace_graph;
pub mod theme;
pub mod todo;
pub mod tools;
pub mod url_read;
pub mod url_router;
pub mod version_check;
pub mod workspace_trust;
pub mod worktree_iso;

pub use error::{Error, Result as PiResult};