//! Backend-neutral contracts for Pi secret storage.
//!
//! This crate intentionally contains no OS, filesystem, keychain, or agent
//! integration. Platform adapters own secret material; callers exchange only
//! typed references and opaque envelopes across this boundary.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Logical owner/visibility boundary for a secret.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretScope {
    Session,
    User,
    Project,
    Provider,
}

/// Stable, non-sensitive identifier for a secret held by a backend.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecretReference {
    pub scope: SecretScope,
    pub name: String,
}

impl SecretReference {
    #[must_use]
    pub fn new(scope: SecretScope, name: impl Into<String>) -> Self {
        Self {
            scope,
            name: name.into(),
        }
    }
}

impl fmt::Display for SecretReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{}", self.scope, self.name)
    }
}

/// Opaque encrypted payload exchanged between a backend and its consumer.
/// The core never interprets ciphertext or chooses an encryption algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretEnvelope {
    pub version: u16,
    pub algorithm: String,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub associated_data: Vec<u8>,
}

impl SecretEnvelope {
    #[must_use]
    pub fn new(
        version: u16,
        algorithm: impl Into<String>,
        nonce: Vec<u8>,
        ciphertext: Vec<u8>,
        associated_data: Vec<u8>,
    ) -> Self {
        Self {
            version,
            algorithm: algorithm.into(),
            nonce,
            ciphertext,
            associated_data,
        }
    }
}

/// Backend-neutral secret operation errors. Adapters may preserve richer
/// diagnostics internally, but must not expose secret values through errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretError {
    NotFound(SecretReference),
    InvalidReference,
    UnsupportedEnvelope,
    Backend(String),
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(reference) => write!(f, "secret not found: {reference}"),
            Self::InvalidReference => f.write_str("invalid secret reference"),
            Self::UnsupportedEnvelope => f.write_str("unsupported secret envelope"),
            Self::Backend(message) => write!(f, "secret backend error: {message}"),
        }
    }
}

impl std::error::Error for SecretError {}

/// Abstract secret backend. Implementations live in coding-agent or a
/// platform-specific crate; this trait is safe to use from agent code.
pub trait SecretBackend {
    fn get(&self, reference: &SecretReference) -> Result<SecretEnvelope, SecretError>;
    fn put(
        &mut self,
        reference: SecretReference,
        envelope: SecretEnvelope,
    ) -> Result<(), SecretError>;
    fn delete(&mut self, reference: &SecretReference) -> Result<(), SecretError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_and_envelopes_round_trip_without_secret_material() {
        let reference = SecretReference::new(SecretScope::Provider, "openai/api-key");
        let envelope = SecretEnvelope::new(1, "aes-256-gcm", vec![1, 2], vec![3, 4], vec![]);
        let encoded =
            serde_json::to_vec(&(reference.clone(), envelope.clone())).expect("serialize");
        let decoded: (SecretReference, SecretEnvelope) =
            serde_json::from_slice(&encoded).expect("deserialize");
        assert_eq!(decoded, (reference, envelope));
    }

    #[test]
    fn backend_contract_can_be_implemented_without_platform_dependencies() {
        struct MemoryBackend(Option<SecretEnvelope>);
        impl SecretBackend for MemoryBackend {
            fn get(&self, reference: &SecretReference) -> Result<SecretEnvelope, SecretError> {
                self.0
                    .clone()
                    .ok_or_else(|| SecretError::NotFound(reference.clone()))
            }
            fn put(
                &mut self,
                _: SecretReference,
                envelope: SecretEnvelope,
            ) -> Result<(), SecretError> {
                self.0 = Some(envelope);
                Ok(())
            }
            fn delete(&mut self, _: &SecretReference) -> Result<(), SecretError> {
                self.0 = None;
                Ok(())
            }
        }
        let reference = SecretReference::new(SecretScope::Session, "test");
        let mut backend = MemoryBackend(None);
        assert!(matches!(
            backend.get(&reference),
            Err(SecretError::NotFound(_))
        ));
        backend
            .put(
                reference.clone(),
                SecretEnvelope::new(1, "test", vec![], vec![1], vec![]),
            )
            .expect("put");
        assert!(backend.get(&reference).is_ok());
    }
}
