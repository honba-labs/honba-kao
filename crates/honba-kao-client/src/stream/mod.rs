//! Server-Sent Events (SSE) connections for live user order state, margin
//! breach warnings, and AI thought steps.
//!
//! Streams deliver messages through a channel (`SseStream`) so consumers can
//! bind them to Leptos reactive signals rather than raw closures.

use futures_channel::mpsc::UnboundedReceiver;

pub mod agent_stream;
pub mod orders_stream;
pub mod risk_stream;

pub use agent_stream::{AgentStep, connect_agent_stream};
pub use orders_stream::connect_orders_stream;
pub use risk_stream::connect_risk_stream;

/// A headless SSE stream: the underlying `EventSource` plus a channel receiver
/// that yields deserialized messages as they arrive.
pub struct SseStream<T> {
    pub event_source: web_sys::EventSource,
    pub receiver: UnboundedReceiver<T>,
}
