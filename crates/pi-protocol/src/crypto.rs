//! Protocol-level crypto value encoding and error classification.
//!
//! This module deliberately contains no QuickJS, filesystem, or platform
//! dependencies.  Host runtimes can use these contracts while retaining
//! ownership of key storage and cryptographic operations.

/// Maximum derived-key output accepted by the host boundary.
pub const KDF_MAX_OUTPUT_BYTES: usize = 1_048_576;

/// Stable classification for errors crossing the crypto hostcall seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoErrorClass {
    InvalidInput,
    UnsupportedAlgorithm,
    EntropyUnavailable,
    AuthenticationFailed,
    DerivationFailed,
}

/// Error contract shared by crypto backends and runtime adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoError {
    class: CryptoErrorClass,
    message: String,
}

impl CryptoError {
    pub fn new(class: CryptoErrorClass, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(CryptoErrorClass::InvalidInput, message)
    }
    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self::new(CryptoErrorClass::InvalidInput, message)
    }
    pub fn authentication_failed() -> Self {
        Self::new(
            CryptoErrorClass::AuthenticationFailed,
            "crypto authentication failed",
        )
    }
    pub fn class(&self) -> CryptoErrorClass {
        self.class
    }
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CryptoError {}

/// Runtime-independent key backend contract. Implementations own key material.
pub trait KeyBackend {
    type Key;
    fn load_private_key(&self, encoded: &[u8]) -> Result<Self::Key, CryptoError>;
    fn load_public_key(&self, encoded: &[u8]) -> Result<Self::Key, CryptoError>;
}

/// Runtime-independent cryptographic operation contract.
pub trait CryptoBackend {
    fn aes_gcm_encrypt(
        &self,
        algorithm: &str,
        key: &[u8],
        iv: &[u8],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, CryptoError>;
    fn aes_gcm_decrypt(
        &self,
        algorithm: &str,
        key: &[u8],
        iv: &[u8],
        aad: &[u8],
        ciphertext_and_tag: &[u8],
    ) -> Result<Vec<u8>, CryptoError>;
    fn ed25519_sign(&self, private_key: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn ed25519_verify(
        &self,
        public_key: &[u8],
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, CryptoError>;
}

pub fn validate_aes_gcm_key(algorithm: &str, key_len: usize) -> Result<(), CryptoError> {
    let expected = match algorithm {
        "aes-128-gcm" => 16,
        "aes-256-gcm" => 32,
        _ => {
            return Err(CryptoError::new(
                CryptoErrorClass::UnsupportedAlgorithm,
                "unsupported cipher algorithm",
            ));
        }
    };
    if key_len != expected {
        return Err(CryptoError::invalid_key(format!(
            "{algorithm} key must be exactly {expected} bytes"
        )));
    }
    Ok(())
}

pub fn validate_kdf_output_len(len: usize) -> Result<(), CryptoError> {
    if len == 0 {
        return Err(CryptoError::invalid_input(
            "derived key length must be positive",
        ));
    }
    if len > KDF_MAX_OUTPUT_BYTES {
        return Err(CryptoError::invalid_input(
            "derived key length exceeds maximum",
        ));
    }
    Ok(())
}

/// Output encodings exposed by the crypto protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoEncoding {
    Hex,
    Base64,
}

impl CryptoEncoding {
    /// Parse the Node-compatible encoding names used by the shim.
    pub fn parse(name: &str) -> Self {
        match name {
            "base64" => Self::Base64,
            _ => Self::Hex,
        }
    }
}

/// Encode bytes using the protocol's requested output encoding.
pub fn encode_output(bytes: &[u8], encoding: &str) -> String {
    match CryptoEncoding::parse(encoding) {
        CryptoEncoding::Base64 => {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(bytes)
        }
        CryptoEncoding::Hex => hex_lower(bytes),
    }
}

/// Convert bytes to lowercase hexadecimal.
pub fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_error_preserves_stable_class() {
        let error = CryptoError::authentication_failed();
        assert_eq!(error.class(), CryptoErrorClass::AuthenticationFailed);
        assert_eq!(error.to_string(), "crypto authentication failed");
    }

    #[test]
    fn aes_and_kdf_limits_are_validated_at_the_protocol_boundary() {
        assert!(validate_aes_gcm_key("aes-128-gcm", 16).is_ok());
        assert_eq!(
            validate_aes_gcm_key("aes-256-gcm", 16),
            Err(CryptoError::invalid_key(
                "aes-256-gcm key must be exactly 32 bytes"
            ))
        );
        assert_eq!(
            validate_kdf_output_len(0),
            Err(CryptoError::invalid_input(
                "derived key length must be positive"
            ))
        );
    }

    #[test]
    fn encodes_hex_and_base64_with_stable_fallback() {
        assert_eq!(encode_output(&[0xde, 0xad], "hex"), "dead");
        assert_eq!(encode_output(b"hello", "base64"), "aGVsbG8=");
        assert_eq!(encode_output(&[0xff], "unknown"), "ff");
    }

    #[test]
    fn hex_lower_is_canonical() {
        assert_eq!(hex_lower(&[0x00, 0xab, 0xff]), "00abff");
    }

    #[test]
    fn error_classes_are_distinct_protocol_values() {
        assert_ne!(
            CryptoErrorClass::InvalidInput,
            CryptoErrorClass::EntropyUnavailable
        );
        assert_ne!(
            CryptoErrorClass::AuthenticationFailed,
            CryptoErrorClass::DerivationFailed
        );
    }
}
