//! Round-trip and bound tests for the JSON-RPC framing primitives.

use std::io::BufReader;

use pi_jsonrpc::{
    encode_frame, find_subslice, parse_content_length, read_frame, read_frame_with_scratch,
    PublicTailBuffer, TailBuffer,
};
use serde_json::json;

#[test]
fn encode_frame_writes_content_length_header() {
    let payload = json!({"jsonrpc": "2.0", "id": 1, "result": "ok"});
    let frame = encode_frame(&payload);
    let header_end = frame
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("frame must end headers with CRLF CRLF");
    let header = std::str::from_utf8(&frame[..header_end]).expect("header is ascii");
    let body = &frame[header_end + 4..];
    assert!(header.starts_with("Content-Length: "));
    let declared_len: usize = header
        .trim_start_matches("Content-Length: ")
        .parse()
        .expect("declared length parses");
    assert_eq!(declared_len, body.len());
    let decoded: serde_json::Value =
        serde_json::from_slice(body).expect("body is valid JSON");
    assert_eq!(decoded, payload);
}

#[test]
fn read_frame_round_trips_with_scratch() {
    let payload = json!({"method": "ping"});
    let frame = encode_frame(&payload);
    let mut reader = BufReader::new(frame.as_slice());
    let mut scratch = Vec::new();
    let read = read_frame_with_scratch(&mut reader, &mut scratch)
        .expect("read ok")
        .expect("some frame");
    assert_eq!(read, payload);
    // Scratch should be empty after a complete read.
    assert!(scratch.is_empty());
}

#[test]
fn read_frame_returns_none_on_clean_eof() {
    let empty: &[u8] = &[];
    let outcome = read_frame(&mut BufReader::new(empty)).expect("eof is ok");
    assert!(outcome.is_none());
}

#[test]
fn parse_content_length_rejects_missing_header() {
    let err = parse_content_length(b"\r\n\r\n").expect_err("no content-length");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn find_subslice_finds_needle() {
    let hay = b"prefix\r\n\r\nbody";
    assert_eq!(find_subslice(hay, b"\r\n\r\n"), Some(6));
    assert_eq!(find_subslice(hay, b"missing"), None);
    assert_eq!(find_subslice(b"", b"x"), None);
}

#[test]
fn tail_buffer_caps_and_preserves_tail() {
    let mut buf = TailBuffer::new(8);
    buf.push("hello");
    assert_eq!(buf.len(), 5);
    buf.push("world");
    // Combined "helloworld" = 10 bytes, cap 8 → drain to last 8 chars.
    assert_eq!(buf.len(), 8);
    // Last 8 chars of "helloworld" are "lloworld".
    assert_eq!(buf.tail(), "lloworld");
}

#[test]
fn public_tail_buffer_default_caps_at_32kib() {
    let mut buf = PublicTailBuffer::new();
    let big = "x".repeat(40 * 1024);
    buf.push(&big);
    assert!(buf.tail().len() <= 32 * 1024);
}
