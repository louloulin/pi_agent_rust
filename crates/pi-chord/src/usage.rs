//! Pure provider usage/quota models and presentation helpers.

#![forbid(unsafe_code)]

use serde::Serialize;
use std::time::Duration;

/// Schema tag for usage rows in JSON output and RPC events.
pub const USAGE_SCHEMA: &str = "pi.usage.v1";
pub const USAGE_CACHE_TTL: Duration = Duration::from_secs(60);
pub const USAGE_FETCH_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderUsage {
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub used: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")] pub limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")] pub remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")] pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub resets_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub detail: Option<String>,
    pub source: String,
    pub fetched_at_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")] pub cache_age_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum UsageStatus {
    Ready(ProviderUsage),
    Unavailable { provider: String, reason: String },
    Error { provider: String, error: String },
}

impl UsageStatus {
    #[must_use]
    pub fn provider(&self) -> &str {
        match self {
            Self::Ready(usage) => &usage.provider,
            Self::Unavailable { provider, .. } | Self::Error { provider, .. } => provider,
        }
    }
}

#[must_use]
pub fn render_usage_text(rows: &[UsageStatus]) -> String {
    if rows.is_empty() { return "No providers with credentials configured. Run /login <provider> first.".to_string(); }
    let mut lines = vec!["Provider usage:".to_string()];
    for row in rows {
        match row {
            UsageStatus::Ready(usage) => {
                let mut parts = Vec::new();
                if let Some(plan) = &usage.plan { parts.push(format!("plan {plan}")); }
                match (usage.used, usage.limit) {
                    (Some(used), Some(limit)) => parts.push(format!("{used:.2} of {limit:.2} used")),
                    (Some(used), None) => parts.push(format!("{used:.2} used")), _ => {}
                }
                if let Some(remaining) = usage.remaining { parts.push(format!("{remaining:.2} remaining")); }
                if let Some(unit) = &usage.unit { parts.push(format!("({unit})")); }
                if let Some(resets) = &usage.resets_at { parts.push(format!("resets {resets}")); }
                if let Some(detail) = &usage.detail { parts.push(format!("— {detail}")); }
                if let Some(age) = usage.cache_age_secs { parts.push(format!("[cached {age}s ago]")); }
                if parts.is_empty() { parts.push("no quota data in response".to_string()); }
                lines.push(format!("  {}: {}", usage.provider, parts.join(" ")));
            }
            UsageStatus::Unavailable { provider, reason } => lines.push(format!("  {provider}: unavailable — {reason}")),
            UsageStatus::Error { provider, error } => lines.push(format!("  {provider}: read failed — {error}")),
        }
    }
    lines.join("\n")
}

#[must_use]
pub fn render_usage_json(rows: &[UsageStatus]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({"schema": USAGE_SCHEMA, "providers": rows})).unwrap_or_else(|_| "{}".to_string())
}
