//! Pure plan data and lifecycle primitives.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanStatus { Draft, Active, Completed, Archived, Cancelled }
impl Default for PlanStatus { fn default() -> Self { Self::Draft } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep { pub id: u32, pub text: String, #[serde(default)] pub done: bool }
impl PlanStep { pub fn new(id: u32, text: impl Into<String>) -> Self { Self { id, text: text.into(), done: false } } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    #[serde(default)] pub status: PlanStatus,
    #[serde(default)] pub steps: Vec<PlanStep>,
}
impl Plan {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self { Self { id: id.into(), title: title.into(), status: PlanStatus::Draft, steps: Vec::new() } }
    pub fn add_step(&mut self, text: impl Into<String>) -> u32 { let id = self.steps.iter().map(|s| s.id).max().unwrap_or(0) + 1; self.steps.push(PlanStep::new(id, text)); id }
    pub fn complete_step(&mut self, id: u32) -> bool { self.steps.iter_mut().find(|s| s.id == id).map_or(false, |s| { s.done = true; true }) }
    pub fn is_complete(&self) -> bool { !self.steps.is_empty() && self.steps.iter().all(|s| s.done) }
    pub fn validate(&self) -> Result<(), PlanValidationError> {
        if self.id.trim().is_empty() { return Err(PlanValidationError::EmptyId); }
        if self.title.trim().is_empty() { return Err(PlanValidationError::EmptyTitle); }
        if self.steps.iter().any(|s| s.text.trim().is_empty()) { return Err(PlanValidationError::EmptyStep); }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanValidationError { EmptyId, EmptyTitle, EmptyStep }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn lifecycle_and_ids() { let mut p = Plan::new("p", "Ship"); assert_eq!(p.add_step("Implement"), 1); assert_eq!(p.add_step("Verify"), 2); assert!(p.complete_step(1)); assert!(p.complete_step(2)); assert!(p.is_complete()); assert!(!p.complete_step(9)); }
    #[test] fn serde_round_trip() { let mut p = Plan::new("p", "Round trip"); p.status = PlanStatus::Active; p.add_step("Test"); let q: Plan = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap(); assert_eq!(p, q); }
    #[test] fn validation() { assert_eq!(Plan::new("", "title").validate(), Err(PlanValidationError::EmptyId)); assert_eq!(Plan::new("id", "").validate(), Err(PlanValidationError::EmptyTitle)); }
}
