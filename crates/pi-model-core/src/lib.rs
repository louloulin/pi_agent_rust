//! Pure model-registry contracts.
//!
//! This crate deliberately contains no filesystem, HTTP, credential, or
//! provider transport code. It is safe to reuse from catalog consumers and
//! keeps runtime integration in `pi-coding-agent`.

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Text,
    Image,
    Video,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
    #[serde(default)]
    pub cache_read: Option<f64>,
    #[serde(default)]
    pub cache_write: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDescriptor {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api: String,
    pub base_url: String,
    pub reasoning: bool,
    pub input: Vec<InputType>,
    pub cost: Option<ModelCost>,
    pub context_window: u32,
    pub max_tokens: u32,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySet {
    pub store: Option<bool>,
    pub developer_role: Option<bool>,
    pub reasoning_effort: Option<bool>,
    pub usage_in_streaming: Option<bool>,
    pub tools: Option<bool>,
    pub streaming: Option<bool>,
    pub parallel_tool_calls: Option<bool>,
}

impl CapabilitySet {
    /// Merge model-specific values over provider defaults.
    pub fn negotiate(model: &Self, provider: &Self) -> Self {
        Self {
            store: model.store.or(provider.store),
            developer_role: model.developer_role.or(provider.developer_role),
            reasoning_effort: model.reasoning_effort.or(provider.reasoning_effort),
            usage_in_streaming: model.usage_in_streaming.or(provider.usage_in_streaming),
            tools: model.tools.or(provider.tools),
            streaming: model.streaming.or(provider.streaming),
            parallel_tool_calls: model.parallel_tool_calls.or(provider.parallel_tool_calls),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionComparison {
    Older,
    Equal,
    Newer,
    Incompatible,
}

/// Compare a candidate version against a required semantic-version range.
pub fn compare_version(candidate: &str, requirement: &str) -> VersionComparison {
    let Ok(version) = Version::parse(candidate.trim()) else {
        return VersionComparison::Incompatible;
    };
    let Ok(requirement) = VersionReq::parse(requirement.trim()) else {
        return VersionComparison::Incompatible;
    };
    if !requirement.matches(&version) {
        return VersionComparison::Older;
    }
    let minimum = requirement.comparators.iter().next().map(|comparator| {
        Version::new(
            comparator.major,
            comparator.minor.unwrap_or(0),
            comparator.patch.unwrap_or(0),
        )
    });
    match minimum {
        Some(minimum) => match version.cmp(&minimum) {
            Ordering::Less => VersionComparison::Older,
            Ordering::Equal => VersionComparison::Equal,
            Ordering::Greater => VersionComparison::Newer,
        },
        None => VersionComparison::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_capabilities_override_provider_defaults() {
        let provider = CapabilitySet {
            tools: Some(true),
            streaming: Some(true),
            ..Default::default()
        };
        let model = CapabilitySet {
            tools: Some(false),
            ..Default::default()
        };
        let negotiated = CapabilitySet::negotiate(&model, &provider);
        assert_eq!(negotiated.tools, Some(false));
        assert_eq!(negotiated.streaming, Some(true));
    }

    #[test]
    fn version_comparison_distinguishes_range_and_invalid_input() {
        assert_eq!(compare_version("1.4.0", ">=1.2.0"), VersionComparison::Newer);
        assert_eq!(compare_version("1.2.0", ">=1.2.0"), VersionComparison::Equal);
        assert_eq!(compare_version("not-a-version", ">=1.2.0"), VersionComparison::Incompatible);
    }
}
