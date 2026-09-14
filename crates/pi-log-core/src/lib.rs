//! Pure structured logging contracts shared by Pi runtimes and sinks.
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const LOG_SCHEMA_VERSION: &str = "pi.ext.log.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPayload { pub schema: String, pub ts: String, pub level: LogLevel, pub event: String, pub message: String, pub correlation: LogCorrelation, #[serde(default, skip_serializing_if = "Option::is_none")] pub source: Option<LogSource>, #[serde(default, skip_serializing_if = "Option::is_none")] pub data: Option<Value> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCorrelation { pub extension_id: String, pub scenario_id: String, #[serde(default, skip_serializing_if = "Option::is_none")] pub session_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub run_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub artifact_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub tool_call_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub slash_command_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub event_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub host_call_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub rpc_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub trace_id: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub span_id: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSource { pub component: LogComponent, #[serde(default, skip_serializing_if = "Option::is_none")] pub host: Option<String>, #[serde(default, skip_serializing_if = "Option::is_none")] pub pid: Option<u32> }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogComponent { Capture, Harness, Runtime, Extension }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel { Debug, Info, Warn, Error }

pub trait LogSink { type Error; fn write(&mut self, payload: &LogPayload) -> Result<(), Self::Error>; }

pub fn validate_log(payload: &LogPayload) -> Result<(), String> {
    if payload.schema != LOG_SCHEMA_VERSION { return Err(format!("Unsupported log schema: {}", payload.schema)); }
    if payload.ts.trim().is_empty() { return Err("Log timestamp is empty".into()); }
    if payload.event.trim().is_empty() { return Err("Log event is empty".into()); }
    if payload.message.trim().is_empty() { return Err("Log message is empty".into()); }
    if payload.correlation.extension_id.trim().is_empty() { return Err("Log correlation extension_id is empty".into()); }
    if payload.correlation.scenario_id.trim().is_empty() { return Err("Log correlation scenario_id is empty".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn payload() -> LogPayload { LogPayload { schema: LOG_SCHEMA_VERSION.into(), ts: "now".into(), level: LogLevel::Info, event: "event".into(), message: "message".into(), correlation: LogCorrelation { extension_id: "ext".into(), scenario_id: "scenario".into(), session_id: None, run_id: None, artifact_id: None, tool_call_id: None, slash_command_id: None, event_id: None, host_call_id: None, rpc_id: None, trace_id: None, span_id: None }, source: None, data: None } }
    #[test] fn validates_contract() { assert!(validate_log(&payload()).is_ok()); }
    #[test] fn rejects_wrong_schema() { let mut p = payload(); p.schema = "other".into(); assert!(validate_log(&p).is_err()); }
}
