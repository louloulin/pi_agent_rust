//! Pure HTML and URL text helpers shared by Pi surfaces.

/// Escape text for insertion into HTML text and attribute contexts.
pub fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Percent-encode one URL query component using RFC 3986 unreserved bytes.
pub fn percent_encode_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push_str("%20"),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Decode a URL query component, accepting both percent escapes and `+` spaces.
pub fn percent_decode_component(value: &str) -> Option<String> {
    if !value.as_bytes().contains(&b'%') && !value.as_bytes().contains(&b'+') {
        return Some(value.to_string());
    }
    let mut out = Vec::with_capacity(value.len());
    let mut bytes = value.as_bytes().iter().copied();
    while let Some(byte) = bytes.next() {
        match byte {
            b'+' => out.push(b' '),
            b'%' => {
                let hi = bytes.next()?;
                let lo = bytes.next()?;
                let pair = [hi, lo];
                out.push(u8::from_str_radix(std::str::from_utf8(&pair).ok()?, 16).ok()?);
            }
            other => out.push(other),
        }
    }
    String::from_utf8(out).ok()
}

/// Parse `key=value` query pairs, discarding malformed pairs.
pub fn parse_query_pairs(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|part| !part.trim().is_empty())
        .filter_map(|part| {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            Some((
                percent_decode_component(key.trim())?,
                percent_decode_component(value.trim())?,
            ))
        })
        .collect()
}

/// Append encoded query parameters to a base URL.
pub fn build_url_with_query(base: &str, params: &[(&str, &str)]) -> String {
    let mut url = String::with_capacity(base.len() + 128);
    url.push_str(base);
    url.push('?');
    for (index, (key, value)) in params.iter().enumerate() {
        if index > 0 {
            url.push('&');
        }
        url.push_str(&percent_encode_component(key));
        url.push('=');
        url.push_str(&percent_encode_component(value));
    }
    url
}

/// Extract OAuth `code` and `state` from a callback URL or compact input.
pub fn parse_oauth_code_input(input: &str) -> (Option<String>, Option<String>) {
    let value = input.trim();
    if value.is_empty() {
        return (None, None);
    }
    if let Some((_, query)) = value.split_once('?') {
        let pairs = parse_query_pairs(query.split('#').next().unwrap_or(query));
        return (
            pairs
                .iter()
                .find_map(|(key, value)| (key == "code").then(|| value.clone())),
            pairs
                .iter()
                .find_map(|(key, value)| (key == "state").then(|| value.clone())),
        );
    }
    if let Some((code, state)) = value.split_once('#') {
        return (
            (!code.trim().is_empty()).then(|| code.trim().to_string()),
            (!state.trim().is_empty()).then(|| state.trim().to_string()),
        );
    }
    (Some(value.to_string()), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html_delimiters_and_quotes() {
        assert_eq!(escape_html("<&>\"'"), "&lt;&amp;&gt;&quot;&#39;");
    }

    #[test]
    fn round_trips_encoded_query_components() {
        let encoded = percent_encode_component("hello world & 日本");
        assert_eq!(
            percent_decode_component(&encoded).as_deref(),
            Some("hello world & 日本")
        );
    }

    #[test]
    fn parses_query_and_fragment_oauth_inputs() {
        assert_eq!(
            parse_oauth_code_input("https://example.test/cb?code=a%20b&state=s#ignored"),
            (Some("a b".into()), Some("s".into()))
        );
        assert_eq!(
            parse_oauth_code_input("code#state"),
            (Some("code".into()), Some("state".into()))
        );
    }

    #[test]
    fn builds_encoded_query_url() {
        assert_eq!(
            build_url_with_query("https://example.test/cb", &[("a b", "x&y")]),
            "https://example.test/cb?a%20b=x%26y"
        );
    }
}
