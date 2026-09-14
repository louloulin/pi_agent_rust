//! Protocol-level crypto value encoding and error classification.
//!
//! This module deliberately contains no QuickJS, filesystem, or platform
//! dependencies.  Host runtimes can use these contracts while retaining
//! ownership of key storage and cryptographic operations.

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

/// Stable classification for errors crossing the crypto hostcall seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoErrorClass {
    InvalidInput,
    UnsupportedAlgorithm,
    EntropyUnavailable,
    AuthenticationFailed,
    DerivationFailed,
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
        assert_ne!(CryptoErrorClass::InvalidInput, CryptoErrorClass::EntropyUnavailable);
        assert_ne!(CryptoErrorClass::AuthenticationFailed, CryptoErrorClass::DerivationFailed);
    }
}
