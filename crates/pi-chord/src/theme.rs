//! Pure theme model, color parsing, validation, and terminal detection.

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
pub enum TerminalBackground { Dark, Light }

#[must_use]
pub fn classify_colorfgbg(value: Option<&str>) -> TerminalBackground {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return TerminalBackground::Dark;
    };
    let mut parts = value.split(';');
    let (Some(_fg), Some(second)) = (parts.next(), parts.next()) else {
        return TerminalBackground::Dark;
    };
    let bg = parts.next_back().unwrap_or(second).trim();
    bg.parse::<u32>().map_or(TerminalBackground::Dark, |index| {
        if index < 8 { TerminalBackground::Dark } else { TerminalBackground::Light }
    })
}

#[must_use]
pub fn detect_terminal_background() -> TerminalBackground {
    classify_colorfgbg(std::env::var("COLORFGBG").ok().as_deref())
}

#[must_use]
pub fn parse_hex_color(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.trim().strip_prefix('#')?;
    if hex.len() != 6 || !hex.is_ascii() { return None; }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

pub fn validate_color(field: &str, value: &str) -> pi_error::Result<()> {
    let value = value.trim();
    if value.len() != 7 || !value.starts_with('#') || !value[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(pi_error::Error::validation(format!("Invalid color for {field}: {value}")));
    }
    Ok(())
}
