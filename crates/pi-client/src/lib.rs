//! Phase-2 aggregator mirroring `@earendil-works/pi-client`: client-side
//! integrations (web_remote, web_search, xdev). After Round 17, the
//! `pi-web-remote` leaf has been inlined directly here.

#![forbid(unsafe_code)]

pub mod web_remote;
pub mod web_search;
pub mod xdev;