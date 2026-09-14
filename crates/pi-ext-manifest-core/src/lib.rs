//! Pure extension manifest schema, validation, and API negotiation.
#![forbid(unsafe_code)]
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

pub const MANIFEST_SCHEMA: &str = "pi.ext.manifest.v1";
pub const CAPABILITY_MANIFEST_SCHEMA_V1: &str = "pi.ext.capabilities.v1";
pub const CAPABILITY_MANIFEST_SCHEMA_V2: &str = "pi.ext.capabilities.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionRuntime { Js, #[serde(rename = "native-rust")] NativeRust, Wasm }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
 pub schema: String, pub extension_id: String, #[serde(default)] pub name: String,
 #[serde(default)] pub version: String, #[serde(default)] pub api_version: String,
 pub runtime: ExtensionRuntime, pub entrypoint: String, #[serde(default)] pub capabilities: Vec<String>,
 #[serde(default, skip_serializing_if = "Option::is_none")] pub capability_manifest: Option<CapabilityManifest>,
 #[serde(default, skip_serializing_if = "Option::is_none")] pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityManifest { pub schema: String, pub capabilities: Vec<CapabilityRequirement> }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
 pub capability: String, #[serde(default)] pub methods: Vec<String>, #[serde(default)] pub intents: Vec<String>,
 #[serde(default)] pub connector_classes: Vec<String>, #[serde(default)] pub hostcall_classes: Vec<String>,
 #[serde(default)] pub risk_tier: Option<String>, #[serde(default)] pub scope: Option<CapabilityScope>,
 #[serde(default)] pub provenance: Option<CapabilityProvenance>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityScope { #[serde(default)] pub paths: Option<Vec<String>>, #[serde(default)] pub hosts: Option<Vec<String>>, #[serde(default)] pub env: Option<Vec<String>>, #[serde(default)] pub allowed_tools: Option<Vec<String>> }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityProvenance { pub source: String, pub integrity: CapabilityIntegrityAttestation, pub publisher: CapabilityPublisherAttestation }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityIntegrityAttestation { pub algorithm: String, pub digest: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityPublisherAttestation { pub id: String, pub verification: String }

#[derive(Debug, Clone, PartialEq, Eq)] pub struct ManifestError(pub String);
impl std::fmt::Display for ManifestError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }
impl std::error::Error for ManifestError {}

pub fn validate_manifest(m: &ExtensionManifest) -> Result<(), ManifestError> {
 if m.schema != MANIFEST_SCHEMA { return Err(ManifestError(format!("Unsupported extension manifest schema: {}", m.schema))); }
 if m.extension_id.is_empty() || !m.extension_id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.'|b'_'|b'-')) { return Err(ManifestError("Invalid extension_id".into())); }
 for (label, value) in [("name", &m.name), ("version", &m.version), ("api_version", &m.api_version), ("entrypoint", &m.entrypoint)] { if value.trim().is_empty() { return Err(ManifestError(format!("Extension manifest {label} is empty"))); } }
 if m.entrypoint.starts_with('/') || m.entrypoint.split('/').any(|p| p == "..") { return Err(ManifestError("Extension manifest entrypoint must stay inside root".into())); }
 if let Some(c) = &m.capability_manifest { validate_capability_manifest(c)?; } Ok(())
}
pub fn validate_capability_manifest(m: &CapabilityManifest) -> Result<(), ManifestError> {
 for (i, r) in m.capabilities.iter().enumerate() { if r.capability.trim().is_empty() { return Err(ManifestError(format!("Capability entry {i} is empty"))); } if m.schema == CAPABILITY_MANIFEST_SCHEMA_V1 { continue; } if m.schema != CAPABILITY_MANIFEST_SCHEMA_V2 || r.methods.len() > 0 || r.intents.is_empty() || r.connector_classes.is_empty() || r.hostcall_classes.is_empty() || r.provenance.is_none() { return Err(ManifestError(format!("Capability v2 entry {i} is incomplete"))); } let p = r.provenance.as_ref().unwrap(); if p.integrity.algorithm != "sha256" || p.integrity.digest.len() != 64 || !p.integrity.digest.bytes().all(|b| b.is_ascii_hexdigit()) || p.publisher.id.trim().is_empty() { return Err(ManifestError(format!("Capability v2 entry {i} has invalid provenance"))); } } Ok(())
}
pub fn negotiate_api_version(required: &str, offered: &[&str]) -> Option<String> { let req = VersionReq::parse(required).ok()?; offered.iter().filter_map(|v| Version::parse(v).ok().map(|p| (p, *v))).filter(|(v, _)| req.matches(v)).max_by_key(|(v, _)| v.clone()).map(|(_, raw)| raw.to_string()) }
#[cfg(test)] mod tests { use super::*; #[test] fn negotiation() { assert_eq!(negotiate_api_version(">=1.0, <2.0", &["1.1.0", "2.0.0", "1.5.0"]).as_deref(), Some("1.5.0")); } }
