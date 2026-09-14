//! `pi-mono` is the workspace aggregator crate for `pi.rs`.
//!
//! After Phase-2 modularization this crate re-exports every top-level
//! package crate in the workspace so consumers can address the full
//! `@earendil-works/pi` surface through a single workspace dependency:
//!
//! ```ignore
//! use pi_mono::pi;
//! use pi_mono::coding_agent;
//! use pi_mono::chord;
//! use pi_mono::protocol;
//! ```
//!
//! Phase-1 only re-exported the legacy `pi` crate. Round 22 expanded the
//! surface to mirror `@earendil-works/pi-mono`'s aggregator package by
//! re-exporting every phase-2 package plus the two Rust-only helper
//! crates (`pi-error`, `pi-provider-metadata`) that have no TypeScript
//! counterpart.
//!
//! ## Module ↔ crate mapping
//!
//! | Module | Crate | `@earendil-works/pi` package |
//! |--------|-------|------------------------------|
//! | `pi`             | `pi`             | `pi` |
//! | `ai`             | `pi-ai`          | `@earendil-works/pi-ai` |
//! | `agent_core`     | `pi-agent-core`  | `@earendil-works/pi-agent` (orchestration slice) |
//! | `chord`          | `pi-chord`       | `@earendil-works/pi-chord` |
//! | `client`         | `pi-client`      | `@earendil-works/pi-client` |
//! | `coding_agent`   | `pi-coding-agent`| `@earendil-works/pi-coding-agent` |
//! | `error`          | `pi-error`       | (Rust-only helper) |
//! | `evals`          | `pi-evals`       | `@earendil-works/pi-evals` |
//! | `protocol`       | `pi-protocol`    | `@earendil-works/pi-protocol` |
//! | `provider_metadata` | `pi-provider-metadata` | (Rust-only helper) |
//! | `server`         | `pi-server`      | `@earendil-works/pi-server` |
//! | `session_backends` | `pi-session-backends` | `@earendil-works/pi-session-backends` |
//! | `telemetry`      | `pi-telemetry`   | `@earendil-works/pi-telemetry` |
//! | `tui`            | `pi-tui`         | `@earendil-works/pi-tui` |

#![forbid(unsafe_code)]

pub use pi;

// Phase-2 top-level packages (mirror @earendil-works/pi-mono).
pub use pi_ai as ai;
pub use pi_agent_core as agent_core;
pub use pi_chord as chord;
pub use pi_client as client;
pub use pi_coding_agent as coding_agent;
pub use pi_evals as evals;
pub use pi_protocol as protocol;
pub use pi_server as server;
pub use pi_session_backends as session_backends;
pub use pi_telemetry as telemetry;
pub use pi_tui as tui;

// Rust-only helpers.
pub use pi_error as error;
// Note: `pi-provider-metadata` exists as an empty stub directory but has
// no Cargo.toml yet, so the aggregator cannot re-export it. The
// canonical home is `pi_ai::provider_metadata`, accessible through the
// `ai` module above.
