//! Pure theme logic shared by Pi front ends.
//!
//! This crate deliberately has no filesystem or TUI dependencies. Loading theme
//! files and converting the model to concrete renderer styles belongs to the
//! coding-agent crate; this crate is safe to reuse from headless consumers.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeColors {
    pub foreground: String,
    pub background: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub muted: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyntaxColors {
    pub keyword: String,
    pub string: String,
    pub number: String,
    pub comment: String,
    pub function: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiColors {
    pub border: String,
    pub selection: String,
    pub cursor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalBackground {
    Dark,
    Light,
}

/// Classify the background index in a terminal COLORFGBG hint.
#[must_use]
pub fn classify_colorfgbg(value: Option<&str>) -> TerminalBackground {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return TerminalBackground::Dark;
    };
    let mut parts = value.split(';');
    let (Some(_foreground), Some(second)) = (parts.next(), parts.next()) else {
        return TerminalBackground::Dark;
    };
    let background = parts.next_back().unwrap_or(second).trim();
    background.parse::<u32>().map_or(TerminalBackground::Dark, |index| {
        if index < 8 {
            TerminalBackground::Dark
        } else {
            TerminalBackground::Light
        }
    })
}

/// Detect the terminal background without querying the terminal.
#[must_use]
pub fn detect_terminal_background() -> TerminalBackground {
    classify_colorfgbg(std::env::var("COLORFGBG").ok().as_deref())
}

/// Parse a six-digit RGB hex color, accepting surrounding whitespace.
#[must_use]
pub fn parse_hex_color(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.trim().strip_prefix('#')?;
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

/// Validate a theme color field and return a structured configuration error.
pub fn validate_color(field: &str, value: &str) -> pi_error::Result<()> {
    let value = value.trim();
    if value.len() != 7
        || !value.starts_with('#')
        || !value[1..].chars().all(|character| character.is_ascii_hexdigit())
    {
        return Err(pi_error::Error::validation(format!(
            "Invalid color for {field}: {value}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_colorfgbg_forms() {
        assert_eq!(classify_colorfgbg(None), TerminalBackground::Dark);
        assert_eq!(classify_colorfgbg(Some("15;0")), TerminalBackground::Dark);
        assert_eq!(classify_colorfgbg(Some("15;default;15")), TerminalBackground::Light);
        assert_eq!(classify_colorfgbg(Some("invalid")), TerminalBackground::Dark);
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_hex_color(" #A0b1C2 "), Some((160, 177, 194)));
        assert_eq!(parse_hex_color("#12345G"), None);
        assert_eq!(parse_hex_color("#123"), None);
    }

    #[test]
    fn validates_colors() {
        assert!(validate_color("accent", "#abcdef").is_ok());
        assert!(validate_color("accent", "red").is_err());
        assert!(validate_color("accent", "#12345G").is_err());
    }
}
