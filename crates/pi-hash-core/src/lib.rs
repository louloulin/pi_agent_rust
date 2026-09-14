//! Pure hashing and binary encoding primitives shared by pi.rs crates.

use base64::Engine as _;
use sha2::digest::Digest as _;

/// Stateless hashing contract implemented by the supported algorithms.
pub trait HashDigest {
    /// Compute the raw digest for `input`.
    fn digest(input: &[u8]) -> Vec<u8>;

    /// Compute the digest and render it as lower-case hexadecimal.
    fn digest_hex(input: &[u8]) -> String {
        encode_hex(&Self::digest(input))
    }
}

/// SHA-1 digest implementation.
pub struct Sha1;
/// SHA-256 digest implementation.
pub struct Sha256;
/// SHA-384 digest implementation.
pub struct Sha384;
/// SHA-512 digest implementation.
pub struct Sha512;
/// MD5 digest implementation.
pub struct Md5;
/// BLAKE3 digest implementation.
pub struct Blake3;

macro_rules! impl_digest {
    ($ty:ty, $algorithm:path) => {
        impl HashDigest for $ty {
            fn digest(input: &[u8]) -> Vec<u8> {
                <$algorithm>::digest(input).to_vec()
            }
        }
    };
}

impl_digest!(Sha1, sha1::Sha1);
impl_digest!(Sha256, sha2::Sha256);
impl_digest!(Sha384, sha2::Sha384);
impl_digest!(Sha512, sha2::Sha512);
impl_digest!(Md5, md5::Md5);

impl HashDigest for Blake3 {
    fn digest(input: &[u8]) -> Vec<u8> {
        blake3::hash(input).as_bytes().to_vec()
    }
}

/// Hash algorithms supported by the core facade.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Md5,
    Blake3,
}

impl HashAlgorithm {
    /// Parse a conventional, case-insensitive algorithm name.
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().replace(['-', '_'], "").as_str() {
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha384" => Some(Self::Sha384),
            "sha512" => Some(Self::Sha512),
            "md5" => Some(Self::Md5),
            "blake3" => Some(Self::Blake3),
            _ => None,
        }
    }

    /// Return the canonical lower-case algorithm name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
            Self::Md5 => "md5",
            Self::Blake3 => "blake3",
        }
    }
}

/// Compute a digest using the requested algorithm.
pub fn digest(algorithm: HashAlgorithm, input: &[u8]) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Sha1 => Sha1::digest(input),
        HashAlgorithm::Sha256 => Sha256::digest(input),
        HashAlgorithm::Sha384 => Sha384::digest(input),
        HashAlgorithm::Sha512 => Sha512::digest(input),
        HashAlgorithm::Md5 => Md5::digest(input),
        HashAlgorithm::Blake3 => Blake3::digest(input),
    }
}

/// Compute a digest and render it as lower-case hexadecimal.
pub fn digest_hex(algorithm: HashAlgorithm, input: &[u8]) -> String {
    encode_hex(&digest(algorithm, input))
}

/// Encode bytes as lower-case hexadecimal.
pub fn encode_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for &byte in bytes {
        output.push(char::from(LUT[usize::from(byte >> 4)]));
        output.push(char::from(LUT[usize::from(byte & 0x0f)]));
    }
    output
}

/// Decode hexadecimal, rejecting malformed input.
pub fn decode_hex(input: &str) -> Result<Vec<u8>, HexDecodeError> {
    if !input.len().is_multiple_of(2) {
        return Err(HexDecodeError::OddLength);
    }
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = decode_nibble(pair[0]).ok_or(HexDecodeError::InvalidCharacter)?;
        let low = decode_nibble(pair[1]).ok_or(HexDecodeError::InvalidCharacter)?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

/// Errors returned by [`decode_hex`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HexDecodeError {
    OddLength,
    InvalidCharacter,
}

fn decode_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Encode bytes with standard padded base64.
pub fn encode_base64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Decode standard padded or unpadded base64.
pub fn decode_base64(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algorithms_parse_and_normalize() {
        assert_eq!(HashAlgorithm::parse("SHA-256"), Some(HashAlgorithm::Sha256));
        assert_eq!(HashAlgorithm::parse("blake3"), Some(HashAlgorithm::Blake3));
        assert_eq!(HashAlgorithm::parse("unknown"), None);
        assert_eq!(HashAlgorithm::Sha512.as_str(), "sha512");
    }

    #[test]
    fn known_digest_vectors() {
        assert_eq!(
            digest_hex(HashAlgorithm::Sha256, b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert_eq!(
            digest_hex(HashAlgorithm::Md5, b"hello"),
            "5d41402abc4b2a76b9719d911017c592"
        );
        assert_eq!(
            digest_hex(HashAlgorithm::Blake3, b"hello"),
            "ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f"
        );
    }

    #[test]
    fn encoding_round_trips_and_rejects_bad_hex() {
        let bytes = b"hello\0world";
        assert_eq!(decode_hex(&encode_hex(bytes)).unwrap(), bytes);
        assert_eq!(decode_base64(&encode_base64(bytes)).unwrap(), bytes);
        assert_eq!(decode_base64("aGVsbG8").unwrap(), b"hello");
        assert_eq!(decode_hex("abc"), Err(HexDecodeError::OddLength));
        assert_eq!(decode_hex("zz"), Err(HexDecodeError::InvalidCharacter));
    }
}
