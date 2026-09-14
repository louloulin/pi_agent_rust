//! Phase-2 aggregator mirroring `@earendil-works/pi-telemetry`: PMU
//! telemetry, profiler, session metrics. After Round 17, every Phase-1
//! leaf has been inlined directly here.

#![forbid(unsafe_code)]

pub mod pmu_telemetry;
pub mod profiler;
pub mod session_metrics;