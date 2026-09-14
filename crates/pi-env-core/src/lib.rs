//! Pure environment-variable parsing and redaction primitives.
//!
//! Runtime access, agent startup policy, and tool execution remain in
//! `pi-coding-agent`; this crate only contains deterministic helpers.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvValue {
    Value(String),
    Redacted,
}

pub fn parse_or_default<T>(raw: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    raw.trim().parse().unwrap_or(default)
}

pub fn parse_bool(raw: Option<&str>, default: bool) -> bool {
    match raw
        .map(str::trim)
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("1" | "true" | "yes" | "on") => true,
        Some("0" | "false" | "no" | "off") => false,
        _ => default,
    }
}

pub fn is_sensitive_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    [
        "api_key",
        "apikey",
        "access_token",
        "auth_token",
        "password",
        "passwd",
        "client_secret",
        "secret",
        "private_key",
        "refresh_token",
        "token",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

pub fn redact_value(name: &str, value: &str) -> EnvValue {
    if is_sensitive_name(name) {
        EnvValue::Redacted
    } else {
        EnvValue::Value(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_sensitive_environment_names_without_exposing_values() {
        assert!(is_sensitive_name("OPENAI_API_KEY"));
        assert!(is_sensitive_name("oauth_client_secret"));
        assert!(!is_sensitive_name("PI_CACHE_RETENTION"));
        assert_eq!(redact_value("OPENAI_API_KEY", "secret"), EnvValue::Redacted);
        assert_eq!(
            redact_value("PI_CACHE_RETENTION", "short"),
            EnvValue::Value("short".into())
        );
    }
}
