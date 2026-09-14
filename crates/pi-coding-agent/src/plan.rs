//! Plan mode (bd-cv653.3.5): a read-only planning state with an approval gate.
//!
//! While `Planning`, the tool executor rejects any tool whose effects
//! intersect the mutation/process BARRIER set with a structured, model-readable
//! error — reads, searches, and analysis flow freely.
//!
//! The agent ends planning by submitting a structured plan via `submit_plan`;
//! the plan is reviewed (TUI card / `/plan approve|reject` / RPC
//! `approve_plan`), and on approval it becomes a pinned context document for
//! execution turns. `--plan-yolo` / `plan.autoApprove` auto-approves for
//! unattended runs. Every transition is logged as session entries
//! (replay-safe).

use crate::tools::ToolEffects;

pub use pi_agent_core::plan::{PlanMode, PlanState};

/// The `submit_plan` tool (bd-cv653.3.5).
///
/// The agent calls this with the full plan to end planning and request
/// review. Session-host-coupled: the shared [`PlanState`] is created by the
/// agent and handed here at registry extension time (like ask/todo).
pub struct SubmitPlanTool {
    state: PlanState,
    auto_approve: bool,
}

impl SubmitPlanTool {
    #[must_use]
    pub const fn new(state: PlanState, auto_approve: bool) -> Self {
        Self {
            state,
            auto_approve,
        }
    }
}

#[async_trait::async_trait]
#[allow(clippy::unnecessary_literal_bound)]
impl crate::tools::Tool for SubmitPlanTool {
    fn name(&self) -> &str {
        "submit_plan"
    }

    fn label(&self) -> &str {
        "submit_plan"
    }

    fn description(&self) -> &str {
        "Submit a completed plan for user review and exit read-only planning. \
         Call ONLY when the plan is complete: goal, ordered steps, files to \
         touch, and verification. The user approves (execution resumes with \
         the plan pinned as context) or rejects with edits (planning \
         continues). Fails when plan mode is not active."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "plan": {
                    "type": "string",
                    "description": "The full plan: goal, ordered steps, files to touch, and how to verify. Include a `Files:` line listing the paths/globs the plan will modify (e.g. `Files: src/main.rs, src/tools/, tests/*.rs`) — under --plan-yolo only mutations inside that scope are auto-approved."
                }
            },
            "required": ["plan"]
        })
    }

    fn effects(&self) -> ToolEffects {
        // Records session state only; mutates no files — must pass the
        // plan-mode gate (which it is called under by definition).
        ToolEffects::read()
    }

    async fn execute(
        &self,
        _tool_call_id: &str,
        input: serde_json::Value,
        _on_update: Option<Box<dyn Fn(crate::tools::ToolUpdate) + Send + Sync>>,
    ) -> pi_error::Result<crate::tools::ToolOutput> {
        let plan = input
            .get("plan")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if plan.len() < 20 {
            return Ok(crate::tools::ToolOutput {
                content: vec![pi_ai::model::ContentBlock::Text(
                    pi_ai::model::TextContent::new(
                        "Plan is too short to review — include goal, ordered steps, files to touch, and verification.",
                    ),
                )],
                details: None,
                is_error: true,
            });
        }
        if !self.state.submit_plan(plan.clone()) {
            return Ok(crate::tools::ToolOutput {
                content: vec![pi_ai::model::ContentBlock::Text(
                    pi_ai::model::TextContent::new(
                        "submit_plan called outside of plan mode. Enter plan mode first (/plan).",
                    ),
                )],
                details: None,
                is_error: true,
            });
        }
        if self.auto_approve {
            // --plan-yolo / plan.autoApprove (bd-cv653.3.5): skip review; the
            // plan rides back in the tool result so execution continues with
            // it in context immediately.
            let _ = self.state.approve();
            return Ok(crate::tools::ToolOutput {
                content: vec![pi_ai::model::ContentBlock::Text(
                    pi_ai::model::TextContent::new(format!(
                        "Plan auto-approved (plan yolo). Execute it now:\n\n{plan}"
                    )),
                )],
                details: Some(serde_json::json!({"planReview": "auto_approved"})),
                is_error: false,
            });
        }
        Ok(crate::tools::ToolOutput {
            content: vec![pi_ai::model::ContentBlock::Text(
                pi_ai::model::TextContent::new(
                    "Plan submitted for review. Wait for the user's decision: on approval, execute the plan; on rejection, revise it from their feedback.",
                ),
            )],
            details: Some(serde_json::json!({"planReview": "pending"})),
            is_error: false,
        })
    }
}

/// Plan-state behavior is implemented in `pi-agent-core`; these tests cover
/// the tool integration in this crate.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_message_is_model_readable() {
        let message = PlanState::block_message("write");
        assert!(message.contains("PLAN_MODE_BLOCKED"));
        assert!(message.contains("submit_plan"));
        assert!(message.contains("\"write\""));
    }
}
