//! Tool policy and error taxonomy shared by hosts and registries.

use pi_protocol::tool_effects::ToolEffects;
use serde::{Deserialize, Serialize};

/// Coarse authorization decision for a tool invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermission {
    Allow,
    Ask,
    Deny,
}

impl ToolPermission {
    #[must_use]
    pub const fn from_effects(effects: ToolEffects) -> Self {
        if effects.processes() || effects.writes() || effects.appends() {
            Self::Ask
        } else {
            Self::Allow
        }
    }
}

/// Stable error families exposed by tool hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorClass {
    InvalidInput,
    PermissionDenied,
    NotFound,
    Execution,
    Cancelled,
    Internal,
}

impl ToolErrorClass {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::PermissionDenied => "permission_denied",
            Self::NotFound => "not_found",
            Self::Execution => "execution",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutating_tools_require_confirmation_by_default() {
        assert_eq!(
            ToolPermission::from_effects(ToolEffects::read()),
            ToolPermission::Allow
        );
        assert_eq!(
            ToolPermission::from_effects(ToolEffects::write()),
            ToolPermission::Ask
        );
        assert_eq!(ToolErrorClass::PermissionDenied.code(), "permission_denied");
    }
}
