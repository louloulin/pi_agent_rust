//! Runtime-independent MCP protocol and configuration types.
//!
//! Transport implementations, process spawning, trust persistence, and tool
//! registration remain in `pi-coding-agent`; these serde types are safe to
//! share with protocol clients and extensions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP transport selected for a server connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpTransportKind {
    Stdio,
    Http,
    Sse,
}

/// MCP initialization protocol revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpProtocolVersion(pub String);

impl McpProtocolVersion {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }

    #[must_use]
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Capabilities advertised by an MCP peer during initialization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<Value>,
}

/// Configuration data needed to describe an MCP server without starting it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<McpTransportKind>,
}

/// Tool metadata returned by an MCP `tools/list` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default)]
    pub input_schema: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_round_trips_wire_shape() {
        let config = McpServerConfig {
            name: "docs".into(),
            command: Some("mcp-docs".into()),
            args: vec!["--stdio".into()],
            ..Default::default()
        };
        let json = serde_json::to_value(&config).expect("serialize config");
        assert_eq!(json["name"], "docs");
        assert_eq!(json["command"], "mcp-docs");
        assert_eq!(json["args"][0], "--stdio");
        assert_eq!(config, serde_json::from_value(json).expect("deserialize config"));
    }
}
