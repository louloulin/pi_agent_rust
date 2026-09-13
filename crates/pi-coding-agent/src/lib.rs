//! Phase-2 aggregator mirroring `@earendil-works/pi-coding-agent`: the
//! coding-agent CLI surface (tools, session, extensions, persistence,
//! platform glue). This crate is a thin re-export of every related
//! leaf crate; the actual `pi` binary lives at `crates/pi` and depends
//! on the individual leaves directly.

#![forbid(unsafe_code)]

pub use pi_cli::*;
pub use pi_conformance::*;
pub use pi_context_files::*;
pub use pi_crash::*;
pub use pi_crypto_shim::*;
pub use pi_extension_replay::*;
pub use pi_extension_scoring::*;
pub use pi_gallery::*;
pub use pi_markdown_rich::*;
pub use pi_overlay_system::*;
pub use pi_platform::*;
pub use pi_secret_screener::*;
pub use pi_secrets::*;
pub use pi_self_update::*;
pub use pi_stats::*;
pub use pi_status_line::*;
pub use pi_swarm_activity_ledger::*;
pub use pi_swarm_progress_slo::*;
pub use pi_turn_recovery::*;
pub use pi_undo::*;
pub use pi_version::*;
pub use pi_workspace::*;
