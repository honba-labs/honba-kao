//! Namespaced API client. Mirrors `web/src/lib/api/` — one namespace per
//! domain, all reachable from this module (the reference app spreads them into
//! a single `api` object).
//!
//! The namespaces currently return sample data from [`crate::data::demo_data`].
//! The real transport ([`http`]) is already wired to the typed
//! `honba-kao-client` crate, so swapping a namespace to the network is a
//! per-function change.

pub mod brokers;
pub mod config;
pub mod data;
pub mod http;
pub mod portfolio;
pub mod strategies;
pub mod telegram;
pub mod traders;
pub mod wallet;

pub use http::{HttpError, base_url, client};
