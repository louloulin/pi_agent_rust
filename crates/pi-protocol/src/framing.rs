//! JSON-RPC 2.0 wire types and `Content-Length` framing.

use std::io::{BufReader, Read};

use serde::{Serialize, Serializer};
use serde_json::Value;

/// Hard cap on a single JSON-RPC frame body (64 MiB); larger frames are
/// treated as transport corruption and the caller is expected to drop the
/// connection.
pub const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

/// A JSON-RPC error object returned by the server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcErrorObject {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Why a request could not complete.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TransportError {
    /// The server returned a JSON-RPC error object.
    Server(RpcErrorObject),
    /// The transport closed (server exited or pipes broke).
    Closed(String),
    /// Local I/O failure writing to or reading from the server.
    Io(String),
}

impl TransportError {
    /// Machine-readable taxonomy code for logs and tool details.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Server(_) => "LSP_SERVER_ERROR",
            Self::Closed(_) => "LSP_TRANSPORT_CLOSED",
            Self::Io(_) => "LSP_TRANSPORT_IO",
        }
    }

    /// Taxonomy code for the MCP flavor of this transport (same classes,
    /// MCP_ prefix so failures name the right subsystem).
    #[must_use]
    pub fn mcp_code(&self) -> String {
        self.code().replace("LSP_", "MCP_")
    }

    /// Human-readable summary.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Server(err) => format!("server error {}: {}", err.code, err.message),
            Self::Closed(reason) => format!("transport closed: {reason}"),
            Self::Io(reason) => format!("transport I/O error: {reason}"),
        }
    }
}

/// A server notification (method + params), queued for the client layer.
#[derive(Debug, Clone)]
pub struct ServerNotification {
    pub method: String,
    pub params: Value,
}

/// How a spawned server's environment is composed.
///
/// Language servers inherit the ambient environment minus scrubbed vars
/// (toolchains need HOME/PATH). MCP servers are third-party code: an
/// explicit allowlist plus their config `env`, never ambient inheritance.
#[derive(Debug, Clone)]
pub enum EnvPolicy {
    /// Inherit the ambient environment, then remove the named vars.
    InheritAndScrub(&'static [&'static str]),
    /// Start empty, copy only the named ambient vars, then apply `env`.
    Allowlist(&'static [&'static str]),
}

/// Ambient vars copied to MCP server processes (no secrets: paths, locale,
/// terminal, and temp dirs only).
pub const MCP_ENV_ALLOWLIST: &[&str] = &[
    "PATH",
    "HOME",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "TMPDIR",
    "TEMP",
    "TMP",
    "NO_COLOR",
    "TERM",
    "SystemRoot",
    "SYSTEMROOT",
    "APPDATA",
    "LOCALAPPDATA",
    "USERPROFILE",
    "COMSPEC",
];

/// Encode one JSON-RPC message as a `Content-Length` framed payload.
#[must_use]
pub fn encode_frame(body: &Value) -> Vec<u8> {
    let json = serde_json::to_vec(body).unwrap_or_else(|_| b"null".to_vec());
    let mut out = Vec::with_capacity(json.len() + 32);
    out.extend_from_slice(format!("Content-Length: {}\r\n\r\n", json.len()).as_bytes());
    out.extend_from_slice(&json);
    out
}

/// Read one framed message from `reader` without consuming any bytes past
/// the frame, so back-to-back frames survive sequential calls (this wrapper
/// has no scratch to carry over-read bytes between calls). Returns `Ok(None)`
/// on clean EOF before any header byte.
pub fn read_frame(reader: &mut BufReader<impl Read>) -> std::io::Result<Option<Value>> {
    // Headers byte-at-a-time (cheap through the BufReader) so nothing beyond
    // this frame is pulled out of the reader.
    let mut header: Vec<u8> = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        if reader.read(&mut byte)? == 0 {
            if header.is_empty() {
                return Ok(None); // clean EOF
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF mid-headers",
            ));
        }
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
        if header.len() > 64 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "headers exceed 64 KiB",
            ));
        }
    }
    let length = parse_content_length(&header)?;
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let value = serde_json::from_slice(&body).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid JSON body: {err}"),
        )
    })?;
    Ok(Some(value))
}

/// Parse the `Content-Length` value out of a raw header block, enforcing the
/// frame-size cap.
pub fn parse_content_length(header_bytes: &[u8]) -> std::io::Result<usize> {
    let headers = String::from_utf8_lossy(header_bytes);
    let mut content_length: Option<usize> = None;
    for line in headers.split("\r\n") {
        if let Some(value) = line
            .split_once(':')
            .map(|(k, v)| (k.trim(), v.trim()))
            .filter(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .map(|(_, v)| v)
        {
            content_length = value.parse::<usize>().ok();
        }
    }
    let length = content_length.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing Content-Length header",
        )
    })?;
    if length > MAX_FRAME_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("frame body {length} bytes exceeds cap {MAX_FRAME_BYTES}"),
        ));
    }
    Ok(length)
}

/// Read one framed message, carrying leftover bytes in `scratch` between
/// calls. `scratch` holds any over-read bytes from the previous frame.
pub fn read_frame_with_scratch(
    reader: &mut BufReader<impl Read>,
    scratch: &mut Vec<u8>,
) -> std::io::Result<Option<Value>> {
    let trace = std::env::var_os("PI_DAP_TRACE").is_some();
    let mut chunk = [0u8; 8192];
    // Phase 1: headers (scratch may already hold some).
    let body_start = loop {
        if let Some(pos) = find_subslice(scratch, b"\r\n\r\n") {
            break pos + 4;
        }
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            return Ok(None); // EOF
        }
        scratch.extend_from_slice(&chunk[..read]);
        if scratch.len() > 64 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "headers exceed 64 KiB",
            ));
        }
    };
    if trace {
        let headers = String::from_utf8_lossy(&scratch[..body_start]);
        eprintln!("[dap-frame] headers: {headers:?}");
    }
    let length = parse_content_length(&scratch[..body_start])?;
    // Phase 2: body (scratch already holds the first bytes after headers).
    let mut body: Vec<u8> = scratch.split_off(body_start);
    while body.len() < length {
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF mid-body",
            ));
        }
        body.extend_from_slice(&chunk[..read]);
    }
    // Preserve over-read bytes for the next frame.
    let leftover = body.split_off(length);
    *scratch = leftover;
    if trace {
        eprintln!(
            "[dap-frame] body {} bytes: {:?}",
            length,
            String::from_utf8_lossy(&body[..length.min(120)])
        );
    }
    let value = serde_json::from_slice(&body).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid JSON body: {err}"),
        )
    })?;
    Ok(Some(value))
}

/// Find the first occurrence of `needle` in `hay`.
#[must_use]
pub fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

// Serialize helper for EnvPolicy (kept private; downstream serialization uses
// the enum directly without going through this helper, but the trait impl is
// here for completeness if external consumers want JSON output).
impl Serialize for EnvPolicy {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::InheritAndScrub(scrub) => {
                use serde::ser::SerializeStruct;
                let mut s = serializer.serialize_struct("EnvPolicy", 2)?;
                s.serialize_field("kind", "inherit_and_scrub")?;
                s.serialize_field("scrub", scrub)?;
                s.end()
            }
            Self::Allowlist(allowlist) => {
                use serde::ser::SerializeStruct;
                let mut s = serializer.serialize_struct("EnvPolicy", 2)?;
                s.serialize_field("kind", "allowlist")?;
                s.serialize_field("allowlist", allowlist)?;
                s.end()
            }
        }
    }
}
