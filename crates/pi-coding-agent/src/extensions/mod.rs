//! Extensions subsystem. The individual `.rs` files in this directory
//! (compatibility, event_coalescer_impl, exec_mediation,
//! extension_manager_impl, fs_connector, native_runtime,
//! native_runtime_experimental, permission_drift, policy_snapshot_tests,
//! protocol, wasm_host) are declared inline in `extensions_api.rs` via
//! `#[path = "extensions/xxx.rs"]`. That keeps `use super::*` inside them
//! resolving to `extensions_api.rs` items while preserving the per-file
//! directory layout for git history.
//!
//! `extensions/tests.rs` and its per-file test split (extensions/tests/*.rs)
//! are also declared inline in `extensions_api.rs`.
//!
//! Re-export the public items that legacy `crate::extensions::X` call sites
//! reference. We enumerate them explicitly rather than using `*` because the
//! inline modules above also live in `extensions_api.rs` and a glob would
//! collide with the module names themselves.

pub use crate::extensions_api::{
    ALL_CAPABILITIES, COMPAT_LEDGER_SCHEMA_VERSION, Capability, CompatCapabilityEvidence,
    CompatLedger, CompatibilityScanner, DangerousCommandClass, DangerousOptInAuditEntry, Error,
    EXTENSION_EVENT_TIMEOUT_MS, EXTENSION_SHORTCUT_BUDGET_MS, EventCoalescer,
    ExtensionAiCompletionRequest, ExtensionDeliverAs, ExtensionEventName, ExtensionHostActions,
    ExtensionLoadSpec, ExtensionManager, ExtensionMessage, ExtensionOverride, ExtensionPolicy,
    ExtensionPolicyMode, ExtensionRegion, ExtensionRepairEvent, ExtensionRuntimeHandle,
    ExtensionSendMessage, ExtensionSendUserMessage, ExtensionSession, ExtensionToolDef,
    ExtensionUiRequest, ExtensionUiResponse, HostCallError, HostCallErrorCode, HostCallPayload,
    HostResultPayload, HostStreamBackpressure, HostStreamChunk, JsExtensionLoadSpec,
    JsExtensionRuntimeHandle, JsExtensionSnapshot, NativeRustExtensionLoadSpec,
    NativeRustExtensionRuntimeHandle, PROTOCOL_VERSION, PolicyCheck, PolicyDecision,
    PolicyProfile, PolicySnapshot, RepairPolicyMode, Result, RuntimeRiskConfig, SecretBrokerPolicy,
    SessionActionOrigin, ToolResultPayload, WasmExtensionHandle, apply_cli_flags,
    classify_dangerous_command, coerce_cli_flag_value, evaluate_exec_mediation,
    extract_slash_command_name, load_extension_manifest, parse_extension_tool_defs,
    resolve_extension_load_spec, safe_canonicalize, strip_unc_prefix,
};
pub(crate) use crate::extensions_api::{
    ExecMediationResult, ExtensionBody, classify_ui_hostcall_error, hash_canonical_json,
    hostcall_params_hash, required_capability_for_host_call_static, SessionActionOriginSource,
    ui_response_value_for_op, validate_host_call,
};

// Round 19 (Option B): wasm_host / policy_snapshot_tests / native_runtime /
// tests are declared inline in `extensions_api.rs` via `#[path = ...]` so
// the existing `use super::*` inside them resolves to the API module. We
// mirror those module names here under cfg gates that match the inline
// declarations so `crate::extensions::wasm_host` (and siblings) keep
// resolving from outside this directory.
#[cfg(feature = "wasm-host")]
pub mod wasm_host;
#[cfg(test)]
pub mod policy_snapshot_tests;
#[cfg(any())]
#[allow(dead_code)]
pub mod native_runtime_experimental;
pub mod native_runtime;
#[cfg(test)]
pub mod tests;