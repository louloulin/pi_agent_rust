//! Pure workspace trust policy types and decision logic.
//!
//! Filesystem scanning, persistence, prompting, and UI remain in the parent
//! application. This crate contains only the portable policy model.

use serde::{Deserialize, Serialize};

/// The trust level assigned to a workspace surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel { Trusted, Untrusted }

impl TrustLevel {
    #[must_use]
    pub const fn is_trusted(self) -> bool { matches!(self, Self::Trusted) }
}

/// A trust rule supplied by an operator or the application environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustRule {
    Cli,
    TrustAll,
    Environment(TrustLevel),
    Stored(TrustLevel),
    Prompt(TrustLevel),
    NonInteractive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustOutcome { pub level: TrustLevel, pub rule: TrustRule }

/// Inputs to the pure trust policy evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustPolicy {
    pub cli_trust: bool,
    pub trust_all_workspaces: bool,
    pub env_override: Option<TrustLevel>,
    pub stored: Option<TrustLevel>,
    pub interactive: bool,
}

impl TrustPolicy {
    /// Evaluate precedence: environment, CLI, global policy, store, then
    /// prompt/non-interactive fallback.
    #[must_use]
    pub const fn evaluate(self, prompt: Option<TrustLevel>) -> TrustOutcome {
        if let Some(level) = self.env_override { return TrustOutcome { level, rule: TrustRule::Environment(level) }; }
        if self.cli_trust { return TrustOutcome { level: TrustLevel::Trusted, rule: TrustRule::Cli }; }
        if self.trust_all_workspaces { return TrustOutcome { level: TrustLevel::Trusted, rule: TrustRule::TrustAll }; }
        if let Some(level) = self.stored { return TrustOutcome { level, rule: TrustRule::Stored(level) }; }
        if self.interactive {
            let level = prompt.unwrap_or(TrustLevel::Untrusted);
            return TrustOutcome { level, rule: TrustRule::Prompt(level) };
        }
        TrustOutcome { level: TrustLevel::Untrusted, rule: TrustRule::NonInteractive }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const fn base() -> TrustPolicy { TrustPolicy { cli_trust: false, trust_all_workspaces: false, env_override: None, stored: None, interactive: false } }

    #[test]
    fn precedence_is_safe_and_explicit() {
        let policy = TrustPolicy { cli_trust: true, trust_all_workspaces: true, env_override: Some(TrustLevel::Untrusted), stored: Some(TrustLevel::Untrusted), interactive: true };
        assert_eq!(policy.evaluate(Some(TrustLevel::Trusted)).rule, TrustRule::Environment(TrustLevel::Untrusted));
    }

    #[test]
    fn stored_and_prompt_are_distinguished() {
        assert_eq!(TrustPolicy { stored: Some(TrustLevel::Trusted), ..base() }.evaluate(None).rule, TrustRule::Stored(TrustLevel::Trusted));
        assert_eq!(TrustPolicy { interactive: true, ..base() }.evaluate(Some(TrustLevel::Trusted)).rule, TrustRule::Prompt(TrustLevel::Trusted));
    }

    #[test]
    fn non_interactive_fails_closed() {
        let outcome = base().evaluate(Some(TrustLevel::Trusted));
        assert_eq!(outcome.level, TrustLevel::Untrusted);
        assert_eq!(outcome.rule, TrustRule::NonInteractive);
    }

    #[test]
    fn levels_serialize_stably() { assert_eq!(serde_json::to_string(&TrustLevel::Trusted).unwrap(), "\"trusted\""); }
}
