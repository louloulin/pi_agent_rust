//! Pure workspace-trust decisions and launch state shared by adapters.
//!
//! Filesystem scanning, persistence, and configuration integration remain in
//! `pi-coding-agent`; these types carry the trust protocol across that seam.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// A recorded (or requested) trust decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustDecision {
    Trusted,
    Untrusted,
}

/// What the workspace declares that could execute local code.
#[derive(Debug, Clone)]
pub struct WorkspaceTrustSurface {
    pub workspace_display: String,
    pub has_project_settings: bool,
    pub package_count: usize,
    pub extension_entries: Vec<String>,
    pub mcp_config_entries: Vec<String>,
    pub digest: String,
}

/// How the effective trust decision was reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustSource {
    NoSurface,
    CliFlag,
    TrustAllConfig,
    EnvOverride,
    Store,
    Prompt,
    NonInteractive,
}

#[derive(Debug)]
pub struct WorkspaceTrustState {
    pub trusted: bool,
    pub source: TrustSource,
    pub surface: Option<WorkspaceTrustSurface>,
}

#[derive(Debug, Clone)]
pub struct TrustInputs {
    pub cli_trust: bool,
    pub trust_all_workspaces: bool,
    pub env_override: Option<String>,
    pub interactive: bool,
}

/// Pure persisted trust decisions keyed by workspace and surface digest.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrustDecisionBook {
    #[serde(default, rename = "workspaces")]
    records: std::collections::BTreeMap<String, TrustDecisionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDecisionRecord {
    pub digest: String,
    pub decision: TrustDecision,
    #[serde(default)]
    pub updated_at: String,
}

impl TrustDecisionBook {
    #[must_use]
    pub fn decision(&self, workspace: &str, digest: &str) -> Option<TrustDecision> {
        self.records.get(workspace)
            .filter(|record| record.digest == digest)
            .map(|record| record.decision)
    }

    pub fn record(&mut self, workspace: impl Into<String>, digest: impl Into<String>, decision: TrustDecision, updated_at: impl Into<String>) {
        self.records.insert(workspace.into(), TrustDecisionRecord {
            digest: digest.into(), decision, updated_at: updated_at.into(),
        });
    }
}
