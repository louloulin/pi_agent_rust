//! Pi - Native AI coding agent CLI (Phase-2 modularization facade)
//!
//! After Phase-2 modularization (see `crates1.md`) and Round 17 (leaf
//! inlining), every implementation module under the legacy `crates/pi/src/`
//! tree has been relocated to one of the 11 Phase-2 aggregator packages
//! (`pi-ai`, `pi-agent-core`, `pi-coding-agent`, `pi-tui`, `pi-telemetry`,
//! `pi-protocol`, `pi-chord`, `pi-client`, `pi-server`,
//! `pi-session-backends`, `pi-evals`).
//!
//! This `pi` crate is a **thin facade**. It re-exports the 11 aggregator
//! packages as nested modules so consumers can opt into the new boundary
//! while binary entry (`src/bin/*`) and integration tests keep resolving.
//!
//! ## Public API policy
//!
//! The `pi` crate is primarily the implementation crate for the `pi` CLI binary.
//! External consumers should treat non-`sdk` modules/types as **unstable**
//! and subject to change. Use [`sdk`] as the stable library-facing surface.
//!
//! Currently intended stable exports:
//! - [`Error`]
//! - [`PiResult`]
//! - [`sdk`] module

#![forbid(unsafe_code)]
// Raised from the default 128 because the RPC command dispatcher's nested
// async blocks exceed it while the compiler proves `Send` for the spawned
// future. It bounds trait-solving depth only; it is not a runtime stack limit.
#![recursion_limit = "256"]
#![allow(dead_code, clippy::unused_async, clippy::unused_async_trait_impl)]
#![cfg_attr(
    test,
    allow(
        unused_variables,
        clippy::assertions_on_constants,
        clippy::match_same_arms,
        clippy::uninlined_format_args,
        clippy::missing_const_for_fn,
        clippy::collapsible_if
    )
)]
#![allow(
    clippy::must_use_candidate,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::wildcard_imports
)]

extern crate self as pi;

/// Serialize unit tests that temporarily change the process-wide current
/// directory. Rust's test runner executes modules concurrently, so separate
/// per-module locks do not prevent one module from observing another module's
/// temporary directory.
#[cfg(test)]
pub(crate) fn test_current_dir_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

// Gap H: jemalloc allocator for allocation-heavy paths.
#[cfg(all(feature = "jemalloc", any(target_os = "linux", target_os = "macos")))]
#[global_allocator]
static GLOBAL_ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// =========================================================================
// Phase-2 modularization facade.
//
// Re-export the 11 Phase-2 aggregator packages as nested modules. Each
// aggregator in turn exposes the modules that were inlined from the
// deleted Phase-1 leaves plus the files relocated from crates/pi/src/.
// =========================================================================

pub mod ai {
    pub use pi_ai::*;
}
pub mod agent_core {
    pub use pi_agent_core::*;
}
pub mod chord {
    pub use pi_chord::*;
}
pub mod client {
    pub use pi_client::*;
}
pub mod coding_agent {
    pub use pi_coding_agent::*;
}
pub mod evals {
    pub use pi_evals::*;
}
pub mod protocol {
    pub use pi_protocol::*;
}
pub mod server {
    pub use pi_server::*;
}
pub mod session_backends {
    pub use pi_session_backends::*;
}
pub mod telemetry {
    pub use pi_telemetry::*;
}
pub mod tui {
    pub use pi_tui::*;
}

// Wildcard re-exports so internal code referencing `use pi::foo::...` from
// the original monolith can still resolve the inlined modules through the
// parent aggregator. Each aggregator already does the same for its leaves.
pub use pi_ai::*;
pub use pi_agent_core::*;
pub use pi_chord::*;
pub use pi_client::*;
pub use pi_coding_agent::*;
pub use pi_evals::*;
pub use pi_protocol::*;
pub use pi_server::*;
pub use pi_session_backends::*;
pub use pi_telemetry::*;
pub use pi_tui::*;

// Stable re-exports named by the public API policy.
pub use pi_coding_agent::Error;
pub use pi_coding_agent::PiResult;
pub use pi_protocol::sdk;

// Conditional re-exports for fuzz harnesses.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub mod fuzz_exports {
    pub use pi_coding_agent::config::Config;
    pub use pi_ai::model::{
        AssistantMessage, ContentBlock, Message, StreamEvent, TextContent, ThinkingContent,
        ToolCall, ToolResultMessage, Usage, UserContent, UserMessage,
    };
    pub use pi_session_backends::session::{Session, SessionEntry, SessionHeader, SessionMessage};
    pub use pi_protocol::sse::{SseEvent, SseParser};
    pub use pi_coding_agent::tools::{fuzz_normalize_dot_segments, fuzz_resolve_path};

    pub use pi_ai::providers::anthropic::fuzz::Processor as AnthropicProcessor;
    pub use pi_ai::providers::azure::fuzz::Processor as AzureProcessor;
    pub use pi_ai::providers::cohere::fuzz::Processor as CohereProcessor;
    pub use pi_ai::providers::gemini::fuzz::Processor as GeminiProcessor;
    pub use pi_ai::providers::openai::fuzz::Processor as OpenAIProcessor;
    pub use pi_ai::providers::openai_responses::fuzz::Processor as OpenAIResponsesProcessor;
    pub use pi_ai::providers::vertex::fuzz::Processor as VertexProcessor;
}