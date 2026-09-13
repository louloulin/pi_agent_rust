//! Magic keywords (bd-cv653.3.6).
//!
//! ## Stage 2 extraction
//!
//! The keyword detection / tokenization / activation surface
//! (`KeywordAction`, `CustomKeyword`, `KeywordSettings`,
//! `KeywordActivation`, `KEYWORD_TELEMETRY_SCHEMA_V1`,
//! `ORCHESTRATE_DIRECTIVE`, `WORKFLOWZ_DIRECTIVE`, `detect`,
//! `directives_for`, `append_session_telemetry`) moved to the
//! `pi-magic-keywords` leaf crate. The leaf defines a
//! `MagicKeywordSink` trait that decouples recording from
//! `crate::session::Session`; this module provides the impl for
//! `Session` so the existing call sites
//! (`agent.rs:14816`, `interactive/agent.rs:56`) keep working unchanged.

#![forbid(unsafe_code)]

pub use pi_magic_keywords::{
    detect, directives_for, append_session_telemetry, CustomKeyword, KeywordAction,
    KeywordActivation, KeywordSettings, MagicKeywordSink, ORCHESTRATE_DIRECTIVE,
    WORKFLOWZ_DIRECTIVE, KEYWORD_TELEMETRY_SCHEMA_V1,
};

impl MagicKeywordSink for crate::session::Session {
    fn append_keyword_entry(&mut self, schema: &str, word: String, action: String) {
        self.append_custom_entry(
            "magic_keyword".to_string(),
            Some(serde_json::json!({
                "schema": schema,
                "word": word,
                "action": action,
            })),
        );
    }
}
