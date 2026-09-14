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
