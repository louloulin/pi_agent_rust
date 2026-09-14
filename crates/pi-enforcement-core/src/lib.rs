//! Pure enforcement policy state machine.
//!
//! This crate intentionally has no runtime, I/O, or UI dependencies. The
//! coding-agent crate adapts its runtime risk and capability-policy types to
//! these deterministic primitives.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementState { Allow = 0, Harden = 1, Prompt = 2, Deny = 3, Terminate = 4 }

impl EnforcementState {
    pub const fn as_str(self) -> &'static str { match self { Self::Allow => "allow", Self::Harden => "harden", Self::Prompt => "prompt", Self::Deny => "deny", Self::Terminate => "terminate" } }
}
impl std::fmt::Display for EnforcementState { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) } }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnforcementScoreBands { pub allow: f64, pub harden: f64, pub prompt: f64, pub deny: f64, pub terminate: f64 }
impl EnforcementScoreBands {
    pub const fn safe() -> Self { Self { allow: 0.0, harden: 0.30, prompt: 0.50, deny: 0.65, terminate: 0.80 } }
    pub const fn balanced() -> Self { Self { allow: 0.0, harden: 0.40, prompt: 0.60, deny: 0.75, terminate: 0.90 } }
    pub const fn permissive() -> Self { Self { allow: 0.0, harden: 0.55, prompt: 0.70, deny: 0.85, terminate: 0.95 } }
    pub fn for_profile(profile: &str) -> Self { match profile { "safe" | "strict" => Self::safe(), "permissive" => Self::permissive(), _ => Self::balanced() } }
    pub fn classify(&self, score: f64) -> EnforcementState { if score >= self.terminate { EnforcementState::Terminate } else if score >= self.deny { EnforcementState::Deny } else if score >= self.prompt { EnforcementState::Prompt } else if score >= self.harden { EnforcementState::Harden } else { EnforcementState::Allow } }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnforcementHysteresis { pub de_escalation_margin: f64, pub cooldown_calls: u32 }
impl Default for EnforcementHysteresis { fn default() -> Self { Self { de_escalation_margin: 0.10, cooldown_calls: 3 } } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementTransition { pub from: EnforcementState, pub to: EnforcementState, pub hysteresis_active: bool, pub raw_band: EnforcementState, pub score: f64, pub cooldown_counter: u32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementStateMachine { state: EnforcementState, bands: EnforcementScoreBands, hysteresis: EnforcementHysteresis, cooldown_counter: u32, evaluation_count: u64 }
impl EnforcementStateMachine {
    pub fn new(profile: &str) -> Self { Self::with_config(EnforcementScoreBands::for_profile(profile), EnforcementHysteresis::default()) }
    pub const fn with_config(bands: EnforcementScoreBands, hysteresis: EnforcementHysteresis) -> Self { Self { state: EnforcementState::Allow, bands, hysteresis, cooldown_counter: 0, evaluation_count: 0 } }
    pub const fn state(&self) -> EnforcementState { self.state }
    pub const fn evaluation_count(&self) -> u64 { self.evaluation_count }
    pub fn evaluate(&mut self, score: f64) -> EnforcementTransition {
        self.evaluation_count += 1; let raw_band = self.bands.classify(score); let previous = self.state;
        if self.state == EnforcementState::Terminate { self.cooldown_counter = 0; return self.transition(previous, raw_band, score, false); }
        if raw_band > self.state { self.state = raw_band; self.cooldown_counter = 0; return self.transition(previous, raw_band, score, false); }
        if raw_band == self.state { self.cooldown_counter = 0; return self.transition(previous, raw_band, score, false); }
        let floor = self.entry_threshold_for(self.state) - self.hysteresis.de_escalation_margin;
        if score < floor { self.cooldown_counter += 1; if self.cooldown_counter >= self.hysteresis.cooldown_calls { self.state = Self::one_level_down(self.state); self.cooldown_counter = 0; return self.transition(previous, raw_band, score, false); } return self.transition(previous, raw_band, score, true); }
        self.cooldown_counter = 0; self.transition(previous, raw_band, score, true)
    }
    fn transition(&self, from: EnforcementState, raw_band: EnforcementState, score: f64, hysteresis_active: bool) -> EnforcementTransition { EnforcementTransition { from, to: self.state, hysteresis_active, raw_band, score, cooldown_counter: self.cooldown_counter } }
    const fn entry_threshold_for(&self, state: EnforcementState) -> f64 { match state { EnforcementState::Allow => self.bands.allow, EnforcementState::Harden => self.bands.harden, EnforcementState::Prompt => self.bands.prompt, EnforcementState::Deny => self.bands.deny, EnforcementState::Terminate => self.bands.terminate } }
    const fn one_level_down(state: EnforcementState) -> EnforcementState { match state { EnforcementState::Allow | EnforcementState::Harden => EnforcementState::Allow, EnforcementState::Prompt => EnforcementState::Harden, EnforcementState::Deny => EnforcementState::Prompt, EnforcementState::Terminate => EnforcementState::Terminate } }
    pub fn merge_with_policy(enforcement: EnforcementState, policy: PolicyDecision) -> EnforcementState { let floor = match policy { PolicyDecision::Allow => EnforcementState::Allow, PolicyDecision::Prompt => EnforcementState::Prompt, PolicyDecision::Deny => EnforcementState::Deny }; enforcement.max(floor) }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision { Allow, Prompt, Deny }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn thresholds_and_terminal_state() { let mut sm = EnforcementStateMachine::new("balanced"); assert_eq!(sm.evaluate(0.8).to, EnforcementState::Deny); sm.evaluate(0.95); assert_eq!(sm.state(), EnforcementState::Terminate); assert_eq!(sm.evaluate(0.0).to, EnforcementState::Terminate); }
    #[test] fn hysteresis_requires_cooldown() { let mut sm = EnforcementStateMachine::new("balanced"); sm.evaluate(0.5); sm.evaluate(0.1); sm.evaluate(0.1); assert_eq!(sm.state(), EnforcementState::Harden); assert_eq!(sm.evaluate(0.1).to, EnforcementState::Allow); }
}
