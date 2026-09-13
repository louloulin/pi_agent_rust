//! Extensions subsystem. The files in this directory are declared as
//! modules here so `extensions_api.rs` can `mod xxx;` them inline (the
//! old Rust 2018 rule that allows inline module declarations to find
//! files in a same-named subdirectory).

pub mod compatibility;
pub mod event_coalescer_impl;
pub mod exec_mediation;
pub mod extension_manager_impl;
pub mod fs_connector;
pub mod native;
pub mod native_runtime;
pub mod native_runtime_experimental;
pub mod permission_drift;
pub mod policy_snapshot_tests;
pub mod protocol;
pub mod tests;
pub mod wasm_host;