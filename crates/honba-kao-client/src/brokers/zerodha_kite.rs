//! Zerodha Kite Connect broker adapter (stub).
//!
//! Establishes the `BrokerClient` interface for Zerodha Kite. Network calls
//! (Kite Connect HTTP + WebSocket) are not wired in this scaffold; every
//! mutating operation returns `BrokerError::NotImplemented` with a clear
//! message so callers can distinguish "adapter present but offline" from a
//! transient failure. Wire the real `kite_connect` / `gloo-net` calls here
//! once broker credentials and network transport are available.

use crate::brokers::{BrokerClient, BrokerError};
use crate::dto::order::{OrderIntent, OrderResponse};
use crate::dto::portfolio::AccountSummary;

/// Zerodha Kite Connect adapter.
///
/// `api_key` is stored for the future network wiring; it is intentionally
/// read-only in this stub scaffold.
#[allow(dead_code)]
pub struct ZerodhaKite {
    api_key: String,
    access_token: Option<String>,
}

impl ZerodhaKite {
    /// Create a new adapter with the given API key. Not connected until an
    /// access token is set (post login flow).
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            access_token: None,
        }
    }

    /// Set the access token obtained from the Kite login flow.
    pub fn set_access_token(&mut self, token: impl Into<String>) {
        self.access_token = Some(token.into());
    }

    fn not_implemented<T>(&self, op: &str) -> Result<T, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: format!("zerodha_kite ({op})"),
        })
    }
}

impl Default for ZerodhaKite {
    fn default() -> Self {
        Self::new("")
    }
}

impl BrokerClient for ZerodhaKite {
    fn name(&self) -> &str {
        "zerodha_kite"
    }

    fn is_connected(&self) -> bool {
        self.access_token.is_some()
    }

    fn submit_order(&mut self, intent: OrderIntent) -> Result<OrderResponse, BrokerError> {
        let _ = intent;
        self.not_implemented("submit_order")
    }

    fn modify_order(
        &mut self,
        order_id: &str,
        intent: OrderIntent,
    ) -> Result<OrderResponse, BrokerError> {
        let _ = (order_id, intent);
        self.not_implemented("modify_order")
    }

    fn cancel_order(&mut self, order_id: &str) -> Result<(), BrokerError> {
        let _ = order_id;
        self.not_implemented("cancel_order")
    }

    fn fetch_portfolio(&mut self) -> Result<AccountSummary, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "zerodha_kite (fetch_portfolio)".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_and_connected_state() {
        let mut broker = ZerodhaKite::new("api_key_123");
        assert_eq!(broker.name(), "zerodha_kite");
        assert!(!broker.is_connected());
        broker.set_access_token("token_abc");
        assert!(broker.is_connected());
    }

    #[test]
    fn operations_return_not_implemented() {
        let mut broker = ZerodhaKite::new("api_key_123");
        broker.set_access_token("token");
        let err = broker
            .submit_order(OrderIntent {
                symbol: "RELIANCE".into(),
                side: "BUY".into(),
                order_type: "LIMIT".into(),
                price: Some(2500.0),
                trigger_price: None,
                quantity: 10.0,
            })
            .unwrap_err();
        assert!(matches!(err, BrokerError::NotImplemented { .. }));
        let err = broker.cancel_order("order_1").unwrap_err();
        assert!(matches!(err, BrokerError::NotImplemented { .. }));
        let err = broker.fetch_portfolio().unwrap_err();
        assert!(matches!(err, BrokerError::NotImplemented { .. }));
    }

    #[test]
    fn default_is_disconnected() {
        let broker = ZerodhaKite::default();
        assert!(!broker.is_connected());
    }
}
