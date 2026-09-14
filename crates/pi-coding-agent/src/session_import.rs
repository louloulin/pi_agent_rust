//! Compatibility facade for foreign session import.
//!
//! The conversion engine lives in `pi-session-backends`; this module adapts
//! the native `Session` store while preserving the historical API path.

pub use pi_session_backends::session_import::{ImportOutcome, ImportSource, IMPORT_SCHEMA};
use std::path::{Path, PathBuf};
use crate::session::{Session, SessionMessage};
use crate::config::Config;
use pi_error::{Error, Result};
use pi_ai::model::Message;

struct NativeSink(Session);
impl pi_session_backends::session_import::SessionImportSink for NativeSink {
    fn set_identity(&mut self, id: &str, provider: &str, model_id: &str, cwd: &str) {
        self.0.header.id = id.to_string();
        self.0.header.provider = Some(provider.to_string());
        self.0.header.model_id = Some(model_id.to_string());
        self.0.header.cwd = cwd.to_string();
    }
    fn append_message(&mut self, message: Message) {
        self.0.append_message(SessionMessage::from(message));
    }
    fn append_custom_entry(&mut self, kind: String, payload: Option<serde_json::Value>) {
        self.0.append_custom_entry(kind, payload);
    }
    fn save(&mut self) -> Result<PathBuf> {
        futures::executor::block_on(async {
            self.0.save().await?;
            self.0.path.clone().ok_or_else(|| Error::tool("import", "session save produced no path"))
        })
    }
}

struct NativeFactory;
impl pi_session_backends::session_import::SessionImportFactory for NativeFactory {
    fn sessions_dir(&self) -> PathBuf { Config::sessions_dir() }
    fn create(&self, target_dir: PathBuf) -> Box<dyn pi_session_backends::session_import::SessionImportSink> {
        Box::new(NativeSink(Session::create_with_dir(Some(target_dir))))
    }
}

pub fn import_claude(path: &Path, target_dir: Option<&Path>) -> Result<ImportOutcome> {
    pi_session_backends::session_import::import_claude_with(path, target_dir, &NativeFactory)
}

pub fn import_codex(path: &Path, target_dir: Option<&Path>) -> Result<ImportOutcome> {
    pi_session_backends::session_import::import_codex_with(path, target_dir, &NativeFactory)
}
