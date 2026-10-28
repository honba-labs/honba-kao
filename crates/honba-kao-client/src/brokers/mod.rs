//! Broker abstraction adapters.
//!
//! Defines the common `BrokerClient` trait and adapter implementations for
//! Zerodha Kite Connect, Angel One SmartAPI, and an in-memory Paper Trading
//! broker. All operations are synchronous and side-effect bounded so they can
//! be unit-tested natively and compiled to `wasm32-unknown-unknown` without a
//! Tokio runtime or network crate. Production adapters layer async network
//! calls (gloo-net / web-sys) behind this same trait.
//!
//! The trait covers order submission, modification, cancellation, and portfolio
//! fetch. Live depth and order state streaming is owned by the `stream` and
//! `websocket` modules (headless SSE/WebSocket transport).

pub mod angel_one;
pub mod paper_wallet;
pub mod zerodha_kite;

pub use angel_one::AngelOne;
pub use paper_wallet::PaperWallet;
pub use zerodha_kite::ZerodhaKite;

use crate::dto::order::{OrderIntent, OrderResponse};
use crate::dto::portfolio::AccountSummary;

/// Common broker client trait implemented by every broker adapter.
pub trait BrokerClient {
    /// Human-readable broker identifier (e.g. "zerodha_kite", "paper_wallet").
    fn name(&self) -> &str;
    /// Whether the adapter currently has an active session.
    fn is_connected(&self) -> bool;
    /// Submit a new order.
    fn submit_order(&mut self, intent: OrderIntent) -> Result<OrderResponse, BrokerError>;
    /// Modify an existing working order.
    fn modify_order(
        &mut self,
        order_id: &str,
        intent: OrderIntent,
    ) -> Result<OrderResponse, BrokerError>;
    /// Cancel a working order.
    fn cancel_order(&mut self, order_id: &str) -> Result<(), BrokerError>;
    /// Fetch the latest account summary.
    fn fetch_portfolio(&mut self) -> Result<AccountSummary, BrokerError>;
}

/// Errors produced by broker adapters.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum BrokerError {
    #[error("broker {broker} operation not implemented")]
    NotImplemented { broker: String },
    #[error("unauthorized: {message}")]
    Unauthorized { message: String },
    #[error("network error: {message}")]
    Network { message: String },
    #[error("order rejected: {message}")]
    Rejected { message: String },
    #[error("serialization error: {message}")]
    Serialization { message: String },
    #[error("unknown error: {message}")]
    Unknown { message: String },
}

/// Re-exported fill type for convenience in adapters that synthesize fills.
pub use crate::dto::order::Fill;
