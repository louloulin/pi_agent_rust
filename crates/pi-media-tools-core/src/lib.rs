//! Pure media values and validation shared by Pi media runtimes.
//!
//! This crate intentionally has no async, filesystem, provider, or UI
//! dependencies. Runtime adapters can turn these values into tool outputs.

use std::fmt;

pub const MAX_IMAGE_FILE_SIZE_BYTES: u64 = 20 * 1024 * 1024;
pub const MAX_TTS_TEXT_CHARS: usize = 4096;
pub const DEFAULT_MEDIA_MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const READ_MEDIA_EXTENSIONS: &[&str] = &["mp4", "webm", "mov", "mp3", "wav", "m4a", "ogg", "flac"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaTransform { InspectImage, GenerateImage, TextToSpeech, ReadMedia }

impl MediaTransform {
    pub const fn tool_name(self) -> &'static str {
        match self { Self::InspectImage => "inspect_image", Self::GenerateImage => "generate_image", Self::TextToSpeech => "tts", Self::ReadMedia => "read_media" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaAsset { pub name: Option<String>, pub mime_type: String, pub data: Vec<u8> }

impl MediaAsset {
    pub fn new(mime_type: impl Into<String>, data: Vec<u8>) -> Self { Self { name: None, mime_type: mime_type.into(), data } }
    #[must_use] pub fn with_name(mut self, name: impl Into<String>) -> Self { self.name = Some(name.into()); self }
    #[must_use] pub const fn size_bytes(&self) -> usize { self.data.len() }
    pub fn validate_size(&self, max_bytes: u64) -> Result<(), MediaError> {
        let size = self.data.len() as u64;
        if size > max_bytes { return Err(MediaError::TooLarge { size, max_bytes }); }
        if size == 0 { return Err(MediaError::Empty); }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaError { Empty, TooLarge { size: u64, max_bytes: u64 }, UnsupportedExtension(String) }

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { Self::Empty => f.write_str("media asset is empty"), Self::TooLarge { size, max_bytes } => write!(f, "media asset is {size} bytes, above the {max_bytes} byte cap"), Self::UnsupportedExtension(ext) => write!(f, "unsupported media extension: .{ext}") }
    }
}
impl std::error::Error for MediaError {}

/// Maps a read_media extension to the MIME spelling used by Gemini inline data.
pub fn media_mime_type_for_extension(ext: &str) -> Option<&'static str> {
    match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
        "mp4" => Some("video/mp4"), "webm" => Some("video/webm"), "mov" => Some("video/mov"), "mp3" => Some("audio/mpeg"), "wav" => Some("audio/wav"), "m4a" => Some("audio/m4a"), "ogg" => Some("audio/ogg"), "flac" => Some("audio/flac"), _ => None,
    }
}

pub fn validate_media_extension(ext: &str) -> Result<&'static str, MediaError> {
    media_mime_type_for_extension(ext).ok_or_else(|| MediaError::UnsupportedExtension(ext.trim_start_matches('.').to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn maps_extensions_case_insensitively() { assert_eq!(media_mime_type_for_extension(".WAV"), Some("audio/wav")); assert_eq!(validate_media_extension("txt"), Err(MediaError::UnsupportedExtension("txt".into()))); }
    #[test] fn validates_asset_size_and_empty_data() { assert_eq!(MediaAsset::new("audio/wav", vec![]).validate_size(10), Err(MediaError::Empty)); assert_eq!(MediaAsset::new("audio/wav", vec![1, 2]).validate_size(1), Err(MediaError::TooLarge { size: 2, max_bytes: 1 })); }
}
