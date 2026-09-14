//! Compatibility facade: time snapshot core lives in `pi-chord`.

use pi_ai::model::{ContentBlock, TextContent};
use pi_error::Result;

use crate::tools::{Tool, ToolEffects, ToolOutput, ToolUpdate};

pub use pi_chord::current_time::TimeSnapshot;

/// The `current_time` built-in tool.
#[derive(Debug, Default, Clone, Copy)]
pub struct CurrentTimeTool;

impl CurrentTimeTool {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
#[allow(clippy::unnecessary_literal_bound)]
impl Tool for CurrentTimeTool {
    fn name(&self) -> &str {
        "current_time"
    }
    fn label(&self) -> &str {
        "current_time"
    }
    fn description(&self) -> &str { "Return the host's current wall-clock time: UTC and local ISO-8601 timestamps, UTC offset, Unix epoch seconds, weekday, and ISO week. Takes no arguments." }
    fn parameters(&self) -> serde_json::Value { serde_json::json!({"type":"object","properties":{},"additionalProperties":false}) }
    fn effects(&self) -> ToolEffects { ToolEffects::read() }
    async fn execute(&self, _tool_call_id: &str, _input: serde_json::Value, _on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>) -> Result<ToolOutput> {
        let snapshot = TimeSnapshot::now();
        Ok(ToolOutput { content: vec![ContentBlock::Text(TextContent::new(snapshot.render_text()))], details: Some(snapshot.details()), is_error: false })
    }
}
