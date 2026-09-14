//! Session-independent plan-mode state and approval gate.
//!
//! The submit tool remains in `pi-coding-agent`, while this state machine is
//! shared by executors and protocol-facing clients without depending on tool
//! implementations.

use pi_protocol::tool_effects::ToolEffects;
use std::sync::{Arc, RwLock};

/// Plan-mode state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlanMode {
    /// Normal operation.
    #[default]
    Off,
    /// Read-only planning; mutations are blocked.
    Planning,
    /// A plan has been submitted and awaits review.
    PendingApproval,
    /// The plan was approved; execution proceeds with the plan pinned.
    Approved,
}

impl PlanMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Planning => "planning",
            Self::PendingApproval => "pending_approval",
            Self::Approved => "approved",
        }
    }
}

/// Shared plan-mode state, held by the agent and submit-plan tool.
#[derive(Debug, Clone, Default)]
pub struct PlanState {
    inner: Arc<RwLock<PlanStateInner>>,
}

#[derive(Debug, Default)]
struct PlanStateInner {
    mode: PlanMode,
    plan: Option<String>,
    previous_model: Option<(String, String)>,
}

impl PlanState {
    #[must_use]
    pub fn new() -> Self { Self::default() }

    #[must_use]
    pub fn mode(&self) -> PlanMode {
        self.inner.read().map_or(PlanMode::Off, |inner| inner.mode)
    }

    /// Enter planning. Returns the previous mode.
    pub fn enter_planning(&self) -> PlanMode {
        let mut inner = self.inner.write().expect("plan state lock");
        let previous = inner.mode;
        inner.mode = PlanMode::Planning;
        previous
    }

    /// Submit a plan for review. Returns false when not planning.
    pub fn submit_plan(&self, plan: String) -> bool {
        let mut inner = self.inner.write().expect("plan state lock");
        if inner.mode != PlanMode::Planning { return false; }
        inner.plan = Some(plan);
        inner.mode = PlanMode::PendingApproval;
        true
    }

    /// Approve the pending plan. Returns the plan text on success.
    pub fn approve(&self) -> Option<String> {
        let mut inner = self.inner.write().expect("plan state lock");
        if inner.mode != PlanMode::PendingApproval { return None; }
        inner.mode = PlanMode::Approved;
        inner.plan.clone()
    }

    /// Reject the pending plan and return to the edit loop.
    pub fn reject(&self) -> bool {
        let mut inner = self.inner.write().expect("plan state lock");
        if inner.mode != PlanMode::PendingApproval { return false; }
        inner.mode = PlanMode::Planning;
        true
    }

    /// Leave plan mode entirely and drop the plan text.
    pub fn exit(&self) {
        let mut inner = self.inner.write().expect("plan state lock");
        inner.mode = PlanMode::Off;
        inner.plan = None;
        inner.previous_model = None;
    }

    /// Reconstruct state for a newly active session, failing closed.
    pub fn reset_for_session(&self, mode: PlanMode) {
        let mut inner = self.inner.write().expect("plan state lock");
        inner.mode = if mode == PlanMode::PendingApproval { PlanMode::Planning } else { mode };
        inner.plan = None;
        inner.previous_model = None;
    }

    #[must_use]
    pub fn plan(&self) -> Option<String> {
        self.inner.read().ok().and_then(|inner| inner.plan.clone())
    }

    pub fn stash_previous_model(&self, provider: &str, model_id: &str) {
        let mut inner = self.inner.write().expect("plan state lock");
        inner.previous_model = Some((provider.to_string(), model_id.to_string()));
    }

    pub fn take_previous_model(&self) -> Option<(String, String)> {
        self.inner.write().expect("plan state lock").previous_model.take()
    }

    /// Whether a tool with these effects may run in the current mode.
    #[must_use]
    pub fn allows_effects(&self, effects: ToolEffects) -> bool {
        match self.mode() {
            PlanMode::Off | PlanMode::Approved => true,
            PlanMode::Planning | PlanMode::PendingApproval =>
                !(effects.writes() || effects.appends() || effects.processes()),
        }
    }

    #[must_use]
    pub fn block_message(tool_name: &str) -> String {
        format!(
            "[PLAN_MODE_BLOCKED] Tool {tool_name:?} is unavailable while planning: plan mode is \
             read-only. Use read/grep/find/ls (and xdev run on read-only tools) to inspect; \
             finish by calling submit_plan with the full plan. The user reviews it and execution \
             resumes on approval."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_and_gate() {
        let state = PlanState::new();
        assert_eq!(state.mode(), PlanMode::Off);
        state.enter_planning();
        assert!(!state.allows_effects(ToolEffects::write()));
        assert!(state.allows_effects(ToolEffects::read()));
        assert!(state.submit_plan("a complete plan".into()));
        assert_eq!(state.approve().as_deref(), Some("a complete plan"));
        assert!(state.allows_effects(ToolEffects::write()));
    }

    #[test]
    fn session_reset_fails_closed() {
        let state = PlanState::new();
        state.enter_planning();
        state.submit_plan("plan text that must not cross sessions".into());
        state.stash_previous_model("provider", "model");
        state.reset_for_session(PlanMode::PendingApproval);
        assert_eq!(state.mode(), PlanMode::Planning);
        assert!(state.plan().is_none());
        assert!(state.take_previous_model().is_none());
    }
}
