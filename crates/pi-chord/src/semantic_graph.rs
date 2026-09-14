//! Pure semantic workspace graph value types and cache algorithms.
//!
//! Filesystem discovery, parsing, redaction, and agent/query integration stay
//! in `pi-coding-agent`; this module contains the dependency-light graph
//! model shared by those adapters.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_CONTEXT_CACHE_TTL_SECONDS: u64 = 15 * 60;

fn default_context_cache_ttl_seconds() -> u64 {
    DEFAULT_CONTEXT_CACHE_TTL_SECONDS
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticWorkspaceGraph {
    pub schema: String,
    pub builder_schema: String,
    pub root: String,
    pub cache_scope: ContextArtifactCacheScope,
    pub cache_ttl_seconds: u64,
    pub nodes: Vec<SemanticGraphNode>,
    pub edges: Vec<SemanticGraphEdge>,
    pub input_fingerprints: Vec<InputFingerprint>,
    pub trace: Vec<GraphBuildTraceEvent>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextArtifactCacheScope {
    pub workspace_identity: String,
    pub branch_identity: String,
    pub session_scope: String,
}

impl Default for ContextArtifactCacheScope {
    fn default() -> Self {
        Self {
            workspace_identity: "workspace-unspecified".to_string(),
            branch_identity: "branch-unspecified".to_string(),
            session_scope: "session-unspecified".to_string(),
        }
    }
}

impl ContextArtifactCacheScope {
    #[must_use]
    pub fn new(
        workspace_identity: impl Into<String>,
        branch_identity: impl Into<String>,
        session_scope: impl Into<String>,
    ) -> Self {
        Self {
            workspace_identity: workspace_identity.into(),
            branch_identity: branch_identity.into(),
            session_scope: session_scope.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBundleBudget {
    pub max_items: usize,
    pub max_bytes: u64,
}

impl Default for ContextBundleBudget {
    fn default() -> Self {
        Self {
            max_items: 24,
            max_bytes: 32 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBundleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bead_id: Option<String>,
    pub changed_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failing_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at_utc: Option<String>,
    #[serde(default = "default_context_cache_ttl_seconds")]
    pub cache_ttl_seconds: u64,
    pub budget: ContextBundleBudget,
}

impl Default for ContextBundleRequest {
    fn default() -> Self {
        Self {
            query: None,
            bead_id: None,
            changed_paths: Vec::new(),
            failing_command: None,
            workspace_id: None,
            branch: None,
            session_id: None,
            generated_at_utc: None,
            cache_ttl_seconds: DEFAULT_CONTEXT_CACHE_TTL_SECONDS,
            budget: ContextBundleBudget::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticContextBundle {
    pub schema: String,
    pub budget: ContextBundleBudget,
    pub selected_items: Vec<ContextBundleItem>,
    pub excluded_items: Vec<ContextBundleExclusion>,
    pub stale_evidence_suppressions: Vec<ContextBundleExclusion>,
    pub redaction_summary: ContextRedactionSummary,
    pub invalidation_policy: ContextBundleInvalidationPolicy,
    pub path_normalization: Vec<ContextPathNormalization>,
    pub suggested_validation_commands: Vec<String>,
    pub estimated_bytes: u64,
    pub estimated_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBundleItem {
    pub node_id: String,
    pub node_type: SemanticNodeType,
    pub source_path: String,
    pub title: String,
    pub reason: String,
    pub score: i64,
    pub estimated_bytes: u64,
    pub estimated_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_status: Option<EvidenceFreshnessStatus>,
    pub redaction_status: RedactionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBundleExclusion {
    pub node_id: String,
    pub node_type: SemanticNodeType,
    pub source_path: String,
    pub title: String,
    pub reason: String,
    pub score: i64,
    pub estimated_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_status: Option<EvidenceFreshnessStatus>,
    pub redaction_status: RedactionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRedactionSummary {
    pub policy_version: String,
    pub overall_status: RedactionStatus,
    pub selected_redacted_nodes: usize,
    pub selected_sensitive_omissions: usize,
    pub suppressed_unsafe_nodes: usize,
    pub redacted_metadata_keys: BTreeSet<String>,
    pub sensitive_path_kinds: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBundleInvalidationPolicy {
    pub policy_version: String,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub input_fingerprint_sha256: String,
    pub cache_ttl_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at_utc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_utc: Option<String>,
    pub invalidates_on: Vec<String>,
    pub cacheable: bool,
}

impl ContextBundleInvalidationPolicy {
    #[must_use]
    pub fn validate_probe(&self, probe: &ContextBundleCacheProbe) -> ContextBundleCacheValidation {
        let mut invalidation_reasons = Vec::new();
        if !self.cacheable {
            invalidation_reasons.push("cache_not_cacheable".to_string());
        }
        if self.workspace_id != probe.workspace_id {
            invalidation_reasons.push("workspace_id_changed".to_string());
        }
        if self.branch != probe.branch {
            invalidation_reasons.push("branch_changed".to_string());
        }
        if optional_cache_scope_value_changed(self.session_id.as_ref(), probe.session_id.as_ref()) {
            invalidation_reasons.push("session_id_changed".to_string());
        }
        if cache_text_values_changed(
            &self.input_fingerprint_sha256,
            &probe.input_fingerprint_sha256,
        ) {
            invalidation_reasons.push("input_fingerprint_changed".to_string());
        }
        match (&self.expires_at_utc, &probe.now_utc) {
            (Some(expires_at), Some(now)) => {
                match (
                    DateTime::parse_from_rfc3339(expires_at),
                    DateTime::parse_from_rfc3339(now),
                ) {
                    (Ok(expires_at), Ok(now)) if now > expires_at => {
                        invalidation_reasons.push("cache_ttl_expired".to_string());
                    }
                    (Ok(_), Ok(_)) => {}
                    _ => invalidation_reasons.push("invalid_cache_timestamp".to_string()),
                }
            }
            _ => invalidation_reasons.push("missing_cache_timestamp".to_string()),
        }

        ContextBundleCacheValidation {
            valid: invalidation_reasons.is_empty(),
            invalidation_reasons,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBundleCacheProbe {
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub input_fingerprint_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub now_utc: Option<String>,
}

fn optional_cache_scope_value_changed(left: Option<&String>, right: Option<&String>) -> bool {
    match (left.map(String::as_str), right.map(String::as_str)) {
        (Some(left), Some(right)) => cache_text_values_changed(left, right),
        (None, None) => false,
        (Some(_), None) | (None, Some(_)) => true,
    }
}

fn cache_text_values_changed(left: &str, right: &str) -> bool {
    !left.as_bytes().iter().eq(right.as_bytes().iter())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBundleCacheValidation {
    pub valid: bool,
    pub invalidation_reasons: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFingerprint {
    pub source_path: String,
    pub normalized_source_path: String,
    pub surface_id: String,
    pub sha256: String,
    pub cache_key_sha256: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime_unix_ns: Option<u64>,
    pub cache_scope: ContextArtifactCacheScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_valid_until_unix_ns: Option<u64>,
    pub cache_status: ContextArtifactCacheStatus,
}

impl InputFingerprint {
    #[must_use]
    pub fn cache_validation(
        &self,
        requested_scope: &ContextArtifactCacheScope,
        now_unix_ns: u64,
    ) -> ContextArtifactCacheStatus {
        if self.cache_status != ContextArtifactCacheStatus::Valid {
            return self.cache_status;
        }
        if self.cache_scope.workspace_identity != requested_scope.workspace_identity {
            return ContextArtifactCacheStatus::WorkspaceMismatch;
        }
        if self.cache_scope.branch_identity != requested_scope.branch_identity {
            return ContextArtifactCacheStatus::BranchMismatch;
        }
        if cache_text_values_changed(
            &self.cache_scope.session_scope,
            &requested_scope.session_scope,
        ) {
            return ContextArtifactCacheStatus::SessionMismatch;
        }
        let Some(cache_valid_until_unix_ns) = self.cache_valid_until_unix_ns else {
            return ContextArtifactCacheStatus::Expired;
        };
        if now_unix_ns > cache_valid_until_unix_ns {
            ContextArtifactCacheStatus::Expired
        } else {
            ContextArtifactCacheStatus::Valid
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticGraphNode {
    pub id: String,
    pub node_type: SemanticNodeType,
    pub source_path: String,
    pub title: String,
    pub stable_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_status: Option<EvidenceFreshnessStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bead_actionability_status: Option<BeadActionabilityStatus>,
    pub redaction_status: RedactionStatus,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticGraphEdge {
    pub id: String,
    pub edge_type: SemanticEdgeType,
    pub source: String,
    pub target: String,
    pub reason: String,
    pub metadata: BTreeMap<String, Value>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticNodeType {
    CodeSymbol,
    FileRegion,
    TestCase,
    DocSection,
    EvidenceArtifact,
    Bead,
    ProviderSurface,
    ValidationCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticEdgeType {
    Contains,
    Defines,
    Exercises,
    Validates,
    CitesEvidence,
    Tracks,
    Blocks,
    DependsOn,
    SuggestsValidation,
    Supersedes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceFreshnessStatus {
    Current,
    HistoricalSnapshot,
    Stale,
    Missing,
    Malformed,
    Uncertified,
    FreshnessUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeadActionabilityStatus {
    ActionableOpen,
    ClaimedInProgress,
    StalledReopenCandidate,
    Blocked,
    ClosedReferenceOnly,
    TombstoneReferenceOnly,
    UnknownFailClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphInputStatus {
    Indexed,
    Missing,
    Unreadable,
    Malformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionStatus {
    None,
    Redacted,
    SensitiveOmitted,
    UnsafeToEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextArtifactCacheStatus {
    Valid,
    Expired,
    WorkspaceMismatch,
    BranchMismatch,
    SessionMismatch,
    MissingFingerprint,
    UnsafePath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedBeadActionability {
    pub status: BeadActionabilityStatus,
    pub planner_may_claim: bool,
    pub reason: String,
}

// Lightweight source parsing helpers shared by graph builders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRustSymbol {
    pub kind: String,
    pub name: String,
}

pub fn parse_rust_symbol(line: &str) -> Option<ParsedRustSymbol> {
    if line.starts_with("//") {
        return None;
    }

    let tokens: Vec<&str> = line
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
        .collect();
    for window in tokens.windows(2) {
        let kind = window[0];
        if matches!(kind, "fn" | "struct" | "enum" | "trait" | "mod") {
            return Some(ParsedRustSymbol {
                kind: kind.to_string(),
                name: window[1].to_string(),
            });
        }
    }
    None
}

pub fn parse_markdown_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let title = trimmed[level..].trim();
    if title.is_empty() {
        return None;
    }
    Some((level, title.to_string()))
}

pub fn extract_evidence_citations(line: &str) -> Vec<String> {
    let mut paths = BTreeSet::new();
    for token in line.split(|ch: char| {
        ch.is_whitespace()
            || matches!(
                ch,
                '`' | '(' | ')' | '[' | ']' | ',' | ';' | '<' | '>' | '"' | '\''
            )
    }) {
        if let Some(path) = normalize_citation_path(token) {
            paths.insert(path);
        }
    }
    paths.into_iter().collect()
}

pub fn normalize_citation_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim_matches(|ch: char| {
        matches!(
            ch,
            '`' | '(' | ')' | '[' | ']' | '<' | '>' | '"' | '\'' | ',' | ';' | ':' | '.'
        )
    });
    let without_anchor = trimmed.split('#').next().unwrap_or(trimmed);
    if is_claim_evidence_path(without_anchor) {
        Some(without_anchor.to_string())
    } else {
        None
    }
}

pub fn is_claim_evidence_path(path: &str) -> bool {
    path == "docs/parity-certification.json"
        || path.starts_with("docs/evidence/") && has_extension(path, "json")
        || path.starts_with("docs/contracts/") && has_extension(path, "json")
        || path.starts_with("tests/perf/reports/") && has_extension(path, "json")
        || path.starts_with("tests/golden_corpus/swarm_claim_readiness/")
            && has_extension(path, "json")
        || path.starts_with("tests/fixtures/vcr/") && has_extension(path, "json")
        || path.starts_with("tests/fixtures/context_artifacts/")
            && (has_extension(path, "json") || has_extension(path, "log"))
}

pub fn claim_surface_for_markdown_line(line: &str) -> &'static str {
    let lower = line.to_ascii_lowercase();
    if lower.contains("historical") || lower.contains("operator evidence only") {
        "historical_snapshot"
    } else if [
        "drop-in",
        "strict replacement",
        "release-facing",
        "release claim",
        "certified",
        "certification",
        "performance claim",
        "perf claim",
        "budget",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        "release_facing"
    } else {
        "documentation"
    }
}
