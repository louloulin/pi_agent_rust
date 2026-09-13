//! Phase-2 aggregator mirroring `@earendil-works/pi-client`:
//! transport-neutral client for remote pi sessions over framed bytes
//! (Rust implementation uses JSON-RPC + HTTP today via `pi-web-remote`).

#![forbid(unsafe_code)]

pub use pi_web_remote::*;
