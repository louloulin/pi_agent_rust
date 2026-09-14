//! Pure MCP configuration contracts shared by discovery and runtime crates.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// MCP protocol version negotiated by the client transport.
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// Where a server definition came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Cli,
    ProjectPi,
    ProjectAgents,
    GlobalPi,
    Foreign,
    Extension,
}

impl Provenance {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::ProjectPi => ".pi",
            Self::ProjectAgents => ".agents",
            Self::GlobalPi => "global",
            Self::Foreign => "foreign",
            Self::Extension => "extension",
        }
    }

    #[must_use]
    pub const fn is_native(self) -> bool {
        !matches!(self, Self::Foreign | Self::Extension)
    }
}

/// One normalized MCP server definition.
#[derive(Debug, Clone)]
pub struct ConfiguredServer {
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub url: Option<String>,
    pub headers: Vec<(String, String)>,
    pub transport_hint: Option<String>,
    pub provenance: Provenance,
    pub source_file: PathBuf,
}

/// A skipped entry surfaced in diagnostics instead of aborting discovery.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigWarning {
    pub source_file: PathBuf,
    pub entry: String,
    pub reason: String,
}

/// Raw server entry used by the JSON/TOML discovery adapter.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawServer {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default, rename = "type")]
    pub transport: Option<String>,
}

#[must_use]
pub fn is_terminal_control(character: char) -> bool {
    character.is_control()
        || matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{2028}' | '\u{2029}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

#[must_use]
pub fn validate_http_header_value(value: &str) -> Result<(), String> {
    if value.len() > 16 * 1024 {
        return Err("HTTP header value exceeds 16384 bytes".to_string());
    }
    if value.chars().any(is_terminal_control) {
        return Err("HTTP header value contains terminal or protocol control characters".to_string());
    }
    Ok(())
}

#[must_use]
pub fn validate_env_value(value: &str) -> Result<(), String> {
    if value.len() > 64 * 1024 {
        return Err("environment value exceeds 65536 bytes".to_string());
    }
    if value.chars().any(is_terminal_control) {
        return Err("environment value contains terminal or process control characters".to_string());
    }
    Ok(())
}

#[must_use]
pub fn validate_server_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 128 || !name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.')) {
        return Err("server name may contain only 1 to 128 ASCII letters, digits, '.', '-', and '_'".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_contract_exposes_version_and_safe_validation() {
        assert_eq!(MCP_PROTOCOL_VERSION, "2025-06-18");
        assert!(validate_server_name("docs-server").is_ok());
        assert!(validate_server_name("bad/name").is_err());
        assert!(validate_http_header_value("line\nforge").is_err());
        assert!(validate_env_value("safe").is_ok());
    }
}
