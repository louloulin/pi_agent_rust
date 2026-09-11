//! Round-trip and bound tests for the PMU telemetry leaf crate.

use pi_pmu_telemetry::{
    PmuOpportunityRanker, PmuRegressionBudget, PmuSample, PMU_TELEMETRY_SCHEMA,
};

fn empty_sample() -> PmuSample {
    PmuSample::default()
}

#[test]
fn schema_identifier_is_stable() {
    assert_eq!(PMU_TELEMETRY_SCHEMA, "pi.pmu.telemetry.v1");
}

#[test]
fn ipc_is_zero_for_zero_cycles() {
    assert_eq!(empty_sample().ipc(), 0.0);
}

#[test]
fn ipc_is_instructions_over_cycles() {
    let mut s = PmuSample::default();
    s.cycles = 1000;
    s.instructions = 2500;
    // 2.5 IPC.
    assert!((s.ipc() - 2.5).abs() < 1e-9);
}

#[test]
fn llc_miss_rate_clamps_to_unit_interval() {
    let mut s = PmuSample::default();
    s.llc_references = 100;
    s.llc_misses = 50;
    assert!((s.llc_miss_rate() - 0.5).abs() < 1e-9);
    // Sanity: bad inputs would never push over 1.0 because clamp() bounds the value.
    s.llc_misses = 10_000;
    assert!(s.llc_miss_rate() <= 1.0);
}

#[test]
fn total_stall_ratio_combines_frontend_and_backend() {
    let mut s = PmuSample::default();
    s.cycles = 1000;
    s.frontend_stall_cycles = 200;
    s.backend_stall_cycles = 300;
    assert!((s.total_stall_ratio() - 0.5).abs() < 1e-9);
}

#[test]
fn default_budget_passes_a_healthy_sample() {
    let mut s = PmuSample::default();
    s.cycles = 10_000;
    s.instructions = 30_000; // IPC = 3.0
    s.llc_references = 1_000;
    s.llc_misses = 10; // 1%
    s.branch_instructions = 1_000;
    s.branch_misses = 10; // 1%
    let verdict = PmuRegressionBudget::default().evaluate(&s);
    assert!(verdict.passed, "violations: {:?}", verdict.violations);
}

#[test]
fn default_budget_fails_low_ipc_sample() {
    let mut s = PmuSample::default();
    s.cycles = 10_000;
    s.instructions = 500; // IPC = 0.05
    let verdict = PmuRegressionBudget::default().evaluate(&s);
    assert!(!verdict.passed);
    assert!(verdict.violations.iter().any(|v| v.starts_with("IPC")));
}

#[test]
fn opportunity_ranker_classifies_memory_bound() {
    let mut s = PmuSample::default();
    s.cycles = 100_000;
    s.llc_references = 1_000;
    s.llc_misses = 400; // 40% — memory-bound threshold is 30%
    let opp = PmuOpportunityRanker::score_candidate("hot_loop", &s);
    assert_eq!(opp.bottleneck_category, "memory_bound_llc");
    assert!(opp.confidence >= 0.9);
    assert!(opp.estimated_speedup >= 1.0);
}

#[test]
fn opportunity_ranker_classifies_compute_bound() {
    let mut s = PmuSample::default();
    s.cycles = 100_000;
    s.instructions = 200_000; // IPC 2.0
    s.llc_references = 0; // no memory signal
    s.branch_instructions = 0; // no branch signal
    s.frontend_stall_cycles = 0;
    s.backend_stall_cycles = 0;
    let opp = PmuOpportunityRanker::score_candidate("compute_kernel", &s);
    assert_eq!(opp.bottleneck_category, "compute_bound");
}

#[test]
fn pmu_sample_round_trips_through_json() {
    let mut s = PmuSample::default();
    s.cycles = 4242;
    s.instructions = 8000;
    s.llc_references = 100;
    s.llc_misses = 5;
    let json = serde_json::to_string(&s).expect("serialize");
    let back: PmuSample = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, s);
}
