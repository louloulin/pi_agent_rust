//! Pure performance-build configuration and metric primitives.
//!
//! This crate deliberately contains no process, filesystem, runtime, or UI
//! code. `pi-coding-agent` owns evidence collection and adapts its harness to
//! these deterministic values.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfMetric {
    BinarySize,
    IdleRss,
    ColdLoad,
}

impl PerfMetric {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BinarySize => "binary_size",
            Self::IdleRss => "idle_rss",
            Self::ColdLoad => "cold_load",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PerfBudget {
    pub metric: PerfMetric,
    pub maximum: f64,
}

impl PerfBudget {
    #[must_use]
    pub const fn new(metric: PerfMetric, maximum: f64) -> Self { Self { metric, maximum } }

    #[must_use]
    pub fn accepts(self, value: f64) -> bool {
        value.is_finite() && self.maximum.is_finite() && self.maximum >= 0.0 && value <= self.maximum
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfConfig {
    pub profile: String,
    pub require_canonical_fingerprint: bool,
    pub budgets: Vec<PerfBudget>,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self { profile: "perf".to_string(), require_canonical_fingerprint: true, budgets: Vec::new() }
    }
}

impl PerfConfig {
    #[must_use]
    pub fn with_budget(mut self, budget: PerfBudget) -> Self {
        self.budgets.push(budget);
        self
    }

    #[must_use]
    pub fn budget_for(&self, metric: PerfMetric) -> Option<PerfBudget> {
        self.budgets.iter().copied().find(|budget| budget.metric == metric)
    }

    #[must_use]
    pub fn accepts(&self, metric: PerfMetric, value: f64) -> bool {
        self.budget_for(metric).is_some_and(|budget| budget.accepts(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_names_are_stable() {
        assert_eq!(PerfMetric::BinarySize.as_str(), "binary_size");
        assert_eq!(PerfMetric::IdleRss.as_str(), "idle_rss");
        assert_eq!(PerfMetric::ColdLoad.as_str(), "cold_load");
    }

    #[test]
    fn config_selects_and_validates_budgets() {
        let config = PerfConfig::default().with_budget(PerfBudget::new(PerfMetric::BinarySize, 10.0));
        assert!(config.accepts(PerfMetric::BinarySize, 10.0));
        assert!(!config.accepts(PerfMetric::BinarySize, 10.1));
        assert!(!config.accepts(PerfMetric::IdleRss, 1.0));
    }

    #[test]
    fn invalid_values_never_pass() {
        assert!(!PerfBudget::new(PerfMetric::ColdLoad, 10.0).accepts(f64::NAN));
        assert!(!PerfBudget::new(PerfMetric::ColdLoad, -1.0).accepts(0.0));
    }
}
