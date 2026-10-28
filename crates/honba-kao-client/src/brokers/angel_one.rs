//! Angel One SmartAPI broker adapter (stub).
//!
//! Establishes the `BrokerClient` interface for Angel One SmartAPI. Network
//! calls (SmartAPI HTTP + SmartStream WebSocket) are not wired in this
//! scaffold; every mutating operation returns `BrokerError::NotImplemented`
//! with a clear message. Wire the real SmartAPI calls here once broker
//! credentials and network transport are available.

use crate::brokers::{BrokerClient, BrokerError};
use crate::dto::order::{OrderIntent, OrderResponse};
use crate::dto::portfolio::AccountSummary;

/// Angel One SmartAPI adapter.
///
/// `api_key` and `client_code` are stored for the future network wiring;
/// they are intentionally read-only in this stub scaffold.
#[allow(dead_code)]
pub struct AngelOne {
    api_key: String,
    client_code: String,
    access_token: Option<String>,
}

impl AngelOne {
    /// Create a new adapter with API key and client code. Not connected until
    /// an access token is set (post login/TOTP flow).
    pub fn new(api_key: impl Into<String>, client_code: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client_code: client_code.into(),
            access_token: None,
        }
    }

    /// Set the access token / JWT obtained from the Angel One login flow.
    pub fn set_access_token(&mut self, token: impl Into<String>) {
        self.access_token = Some(token.into());
    }

    fn not_implemented<T>(&self, op: &str) -> Result<T, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: format!("angel_one ({op})"),
        })
    }
}

impl Default for AngelOne {
    fn default() -> Self {
        Self::new("", "")
    }
}

impl BrokerClient for AngelOne {
    fn name(&self) -> &str {
        "angel_one"
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
            broker: "angel_one (fetch_portfolio)".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_and_connected_state() {
        let mut broker = AngelOne::new("api_key", "client_code");
        assert_eq!(broker.name(), "angel_one");
        assert!(!broker.is_connected());
        broker.set_access_token("jwt_token");
        assert!(broker.is_connected());
    }

    #[test]
    fn operations_return_not_implemented() {
        let mut broker = AngelOne::new("api_key", "client_code");
        broker.set_access_token("jwt");
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
        let err = broker.fetch_portfolio().unwrap_err();
        assert!(matches!(err, BrokerError::NotImplemented { .. }));
    }

    #[test]
    fn default_is_disconnected() {
        let broker = AngelOne::default();
        assert!(!broker.is_connected());
    }
}
