//! `honba-kao-client` — Wire transport, REST client, SSE/WebSocket streams.
//!
//! Headless networking layer owning typed REST clients, Server-Sent Events,
//! auto-reconnecting WebSocket feeds, and domain event envelopes.
//! Zero UI dependencies.

pub mod brokers;
pub mod dto;
pub mod events;
pub mod http;
pub mod stream;
pub mod websocket;

// Convenient re-exports
pub use brokers::*;
pub use dto::*;
pub use events::*;
pub use http::*;
pub use stream::*;
pub use websocket::*;
