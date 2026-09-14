//! Shared tool protocol types and validation.

use async_trait::async_trait;
use futures::future::BoxFuture;
use pi_ai::model::ContentBlock;
use pi_ai::provider::ToolDef;
use pi_error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub use pi_protocol::tool_effects::ToolEffects;

/// Resolver used by tools whose operations are scoped to the active session.
pub type ToolSessionIdFuture = BoxFuture<'static, Option<String>>;
pub type ToolSessionIdResolver = Arc<dyn Fn() -> ToolSessionIdFuture + Send + Sync>;

#[derive(Clone)]
pub struct ToolSessionScope {
    resolver: Arc<Mutex<ToolSessionIdResolver>>,
}

impl ToolSessionScope {
    #[must_use]
    pub fn fixed(session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        let resolver: ToolSessionIdResolver = Arc::new(move || {
            let session_id = session_id.clone();
            Box::pin(async move { Some(session_id) })
        });
        Self {
            resolver: Arc::new(Mutex::new(resolver)),
        }
    }

    pub fn bind(&self, resolver: ToolSessionIdResolver) {
        *self
            .resolver
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = resolver;
    }

    pub async fn session_id(&self) -> Result<String> {
        let resolver = self
            .resolver
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        resolver()
            .await
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| {
                Error::tool(
                    "jobs",
                    "PI_JOBS_SESSION_UNAVAILABLE: current agent session identity is unavailable",
                )
            })
    }
}

impl Default for ToolSessionScope {
    fn default() -> Self {
        Self::fixed(format!("standalone-{}", uuid::Uuid::new_v4().simple()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolOrigin {
    BuiltIn,
    Extension,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOutput {
    pub content: Vec<ContentBlock>,
    pub details: Option<Value>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_error: bool,
}

const fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUpdate {
    pub content: Vec<ContentBlock>,
    pub details: Option<Value>,
}

/// Common contract implemented by executable tools. Concrete implementations
/// remain in `pi-coding-agent`; this crate only owns the protocol boundary.
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn label(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    async fn execute(
        &self,
        tool_call_id: &str,
        input: Value,
        on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
    ) -> Result<ToolOutput>;
    #[must_use]
    fn effects(&self) -> ToolEffects {
        ToolEffects::write()
    }
    fn bind_job_session_scope(&mut self, _scope: ToolSessionScope) {}
    fn origin(&self) -> ToolOrigin {
        ToolOrigin::BuiltIn
    }
}

/// Validate a tool input against its declared JSON Schema.
pub fn validate_input(tool: &dyn Tool, input: &Value) -> Result<()> {
    let schema = tool.parameters();
    let validator = jsonschema::validator_for(&schema).map_err(|error| {
        Error::validation(format!("invalid schema for {}: {error}", tool.name()))
    })?;
    if let Err(error) = validator.validate(input) {
        return Err(Error::validation(format!(
            "invalid input for {}: {error}",
            tool.name()
        )));
    }
    Ok(())
}

/// Convert a tool into the provider-facing schema without exposing its implementation.
pub fn provider_schema(tool: &dyn Tool) -> ToolDef {
    ToolDef {
        name: tool.name().to_string(),
        description: tool.description().to_string(),
        parameters: tool.parameters(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Example;
    #[async_trait]
    impl Tool for Example {
        fn name(&self) -> &str {
            "example"
        }
        fn label(&self) -> &str {
            "Example"
        }
        fn description(&self) -> &str {
            "example"
        }
        fn parameters(&self) -> Value {
            serde_json::json!({"type":"object","required":["name"],"properties":{"name":{"type":"string"}}})
        }
        async fn execute(
            &self,
            _: &str,
            _: Value,
            _: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
        ) -> Result<ToolOutput> {
            unreachable!()
        }
    }
    #[test]
    fn schema_validation_rejects_missing_required_parameter() {
        let error = validate_input(&Example, &serde_json::json!({})).expect_err("missing name");
        assert!(error.to_string().contains("invalid input"));
    }
}
