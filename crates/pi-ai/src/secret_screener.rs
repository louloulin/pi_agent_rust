//! Credential-shape screener used by `pi-memory` and `pi-stream-rules`.
//!
//! The interim floor detector replaces any well-known credential shape
//! (Anthropic / OpenAI keys, GitHub PATs, AWS access keys, PEM private-key
//! headers, Google API keys, Slack tokens) in free-form text with a stable
//! placeholder. Both the memory bank and the TTSR reminder excerpts must
//! never carry raw secrets, so they share this screener.
//!
//! A future consolidation will route every detector through
//! `pi-secrets` (bd-cv653.7.9); today the secret-shape vocabulary here
//! matches what `pi-memory` screens for retention and what `pi-stream-rules`
//! inserts into reminder bodies.

/// Pattern vocabulary: (regex source, replacement placeholder).
const SECRET_PATTERNS: &[(&str, &str)] = &[
    (r"sk-ant-[A-Za-z0-9_\-]{16,}", "[REDACTED_ANTHROPIC_KEY]"),
    (r"sk-[A-Za-z0-9_\-]{16,}", "[REDACTED_OPENAI_KEY]"),
    (r"ghp_[A-Za-z0-9]{20,}", "[REDACTED_GITHUB_PAT]"),
    (r"github_pat_[A-Za-z0-9_]{20,}", "[REDACTED_GITHUB_PAT]"),
    (r"AKIA[0-9A-Z]{16}", "[REDACTED_AWS_ACCESS_KEY]"),
    (
        r"-----BEGIN [A-Z ]*PRIVATE KEY-----",
        "[REDACTED_PRIVATE_KEY]",
    ),
    (r"AIza[0-9A-Za-z_\-]{20,}", "[REDACTED_GOOGLE_API_KEY]"),
    (r"xox[baprs]-[A-Za-z0-9\-]{10,}", "[REDACTED_SLACK_TOKEN]"),
];

fn secret_patterns() -> &'static Vec<(regex::Regex, &'static str)> {
    static PATTERNS: std::sync::LazyLock<Vec<(regex::Regex, &'static str)>> =
        std::sync::LazyLock::new(|| {
            SECRET_PATTERNS
                .iter()
                .map(|(pattern, placeholder)| {
                    (
                        regex::Regex::new(pattern).expect("secret pattern compiles"),
                        *placeholder,
                    )
                })
                .collect()
        });
    &PATTERNS
}

/// Replace any detected credential in `content` with a placeholder.
#[must_use]
pub fn screen_secrets(content: &str) -> String {
    let mut screened = content.to_string();
    for (pattern, placeholder) in secret_patterns() {
        screened = pattern.replace_all(&screened, *placeholder).into_owned();
    }
    screened
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_anthropic_and_openai_key_shapes() {
        let screened = screen_secrets("api key = sk-abcdefghijklmnopqrstuvwxyz");
        assert!(
            !screened.contains("sk-abcdef"),
            "secret must be redacted: {screened}"
        );
        assert!(screened.contains("[REDACTED_OPENAI_KEY]"), "{screened}");
    }

    #[test]
    fn redacts_aws_access_key_shape() {
        let aws = screen_secrets(concat!("aws key AKIA", "IOSFODNN7EXAMPLE here"));
        assert!(aws.contains("[REDACTED_AWS_ACCESS_KEY]"), "{aws}");
    }

    #[test]
    fn leaves_clean_text_untouched() {
        let clean = screen_secrets("nothing secret here");
        assert_eq!(clean, "nothing secret here");
    }

    #[test]
    fn redacts_pem_header() {
        let pem = screen_secrets(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAK...\n-----END RSA PRIVATE KEY-----",
        );
        assert!(pem.contains("[REDACTED_PRIVATE_KEY]"), "{pem}");
        assert!(!pem.contains("BEGIN RSA PRIVATE KEY"), "{pem}");
    }
}
