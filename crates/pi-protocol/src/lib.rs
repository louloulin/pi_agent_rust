//! Phase-2 aggregator mirroring `@earendil-works/pi-protocol`: the
//! protocol layer (jsonrpc, sse, http, acp, sdk, vcr, validation_broker).
//! After Round 17, the `pi-jsonrpc` and `pi-sse` leaves have been inlined
//! directly here.

#![forbid(unsafe_code)]

pub mod acp;
pub mod http;
pub mod jsonrpc;
pub mod rpc;
pub mod sdk;
pub mod sse;
pub mod validation_broker;
pub mod vcr;