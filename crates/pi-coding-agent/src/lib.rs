//! Phase-2 aggregator mirroring `@earendil-works/pi-coding-agent`: the
//! coding-agent CLI surface (tools, session, extensions, persistence,
//! platform glue). After Round 17, every Phase-1 leaf that previously
//! re-exported from this aggregator has been inlined directly here.
//! Round 18 also absorbed session / compaction / sqlite / hostcall /
//! file-lock / swarm modules from pi-session-backends and pi-chord to
//! break two cyclic dependencies. The actual `pi` binary entry is
//! declared in `Cargo.toml` (`[[bin]] name = "pi"`) and lives at
//! `src/main.rs`.

#![forbid(unsafe_code)]

// Inlined from the Phase-1 leaves (Round 17).
pub mod advisor;
pub mod cli;
pub mod conformance;
pub mod context_files;
pub mod crash;
pub mod crypto_shim;
pub mod extension_replay;
pub mod extension_scoring;
pub mod gallery;
pub mod markdown_rich;
pub mod overlay_system;
pub mod platform;
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
pub mod providers;
pub mod gc;
pub mod hub;
pub mod keybindings;
pub mod mcp;
pub mod media_tools;
pub mod model_routing;
pub mod model_selector;
pub mod models;
pub mod perf_build;
pub mod permissions;
pub mod pi_wasm;
pub mod resources;
pub mod security_scan;
pub mod semantic_workspace_graph;
pub mod theme;
pub mod todo;
pub mod tools;
pub mod usage;
pub mod url_read;
pub mod url_router;
pub mod vcr;
pub mod version_check;
pub mod workspace_trust;
pub mod worktree_iso;

// Round 18: modules absorbed from pi-chord and pi-session-backends.
pub mod compaction;
pub mod compaction_worker;
pub mod hostcall_amac;
pub mod hostcall_egraph;
pub mod http_shim;
pub mod migrations;
pub mod session;
pub mod session_import;
pub mod session_index;
pub mod session_picker;
pub mod session_sqlite;
pub mod session_store_v2;
pub mod swarm_flight_recorder;
pub mod swarm_replay;

// Round 18: agent core absorbed from pi-agent-core (breaks
// pi-coding-agent ↔ pi-agent-core cycle by making pi-coding-agent the
// sole owner of the orchestration code).
pub mod agent;
pub mod agent_cx;
pub mod agent_hub;
pub mod handoff;
pub mod memory;
pub mod resource_governor;
pub mod skills_managed;
pub mod subagents;

// Round 18: protocol / rpc / acp / sdk absorbed from pi-protocol.
pub mod acp;
pub mod http;
pub mod jsonrpc;
pub mod rpc;
pub mod sdk;
pub mod validation_broker;

// The extensions/ directory predates Round 17; declared as a single
// `extensions` module so files inside can keep their flat names.
// Submodules (native, native_runtime_experimental, policy_snapshot_tests,
// tests, wasm_host) live in the directory. Compatibility / extension_manager_impl /
// exec_mediation / event_coalescer_impl / fs_connector / permission_drift /
// protocol / native_runtime were moved to crate root in Round 18 and are
// declared flat here so extensions_api.rs can `mod xxx;` them inline.
pub mod extensions;
pub mod connectors;
// Round 18: server-side / client-side / eval / tui modules absorbed from
// pi-server, pi-client, pi-evals, pi-tui.
pub mod package_manager;
pub mod plan;
pub mod jobs;
pub mod github;
pub mod review;
pub mod debug;
pub mod eval;
pub mod xdev;
pub mod lsp;
pub mod autocomplete;
pub mod interactive;
pub mod interactive_ftui;
pub mod terminal_images;
pub mod tui;

pub use pi_error::{Error, Result as PiResult};

// Round 19: re-export the protocol / provider / scheduler / error-hint
// modules from the upstream `pi-ai` and `pi-agent-core` crates so legacy
// `crate::X` call sites inside `extensions/*` continue to compile.
pub use pi_ai::error_hints;
pub use pi_ai::model as model_module;
pub use pi_ai::provider;
pub use pi_ai::provider as provider_module;
pub use pi_agent_core::scheduler;

// Convenience aliases used by inlined extension manager code that
// originally addressed `crate::model::*` and `crate::error::*`.
#[doc(hidden)]
pub mod model {
    pub use crate::model_module::*;
}
#[doc(hidden)]
pub mod error {
    pub use pi_error::{Error, Result, is_retryable_error};
}

// Re-export the web_search module so tool registry wiring finds it.
pub mod web_search;

// Re-export modules that main.rs and tests reference via
// `pi_coding_agent::X`. Round 20: pulled in via `pub use` rather than
// new local modules so binary call sites resolve without forcing a
// re-implementation.
pub use pi_ai::failover;
pub use pi_ai::stream_rules;
pub use pi_ai::token_count;
pub use pi_error::is_retryable_error;
pub use pi_telemetry::profiler;

// `web_remote` already lives in this crate; declare it for module
// resolution.
pub mod web_remote;