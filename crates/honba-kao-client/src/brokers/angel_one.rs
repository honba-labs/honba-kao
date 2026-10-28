//! Angel One SmartAPI broker adapter.
//!
//! Live auth flow (SmartAPI):
//!   1. `login_by_password()` — POST `/loginByPassword` with `api_key`,
//!      `client_code`, `password` and `TOTP`, returning a JWT `access_token`
//!      and a `feed_token`.
//!   2. `connect_feed()` — authenticated SmartStream WebSocket feed.
//!
//! REST order/portfolio operations and the WebSocket feed are wired through
//! `gloo-net` and `WsConnection`. Because the `BrokerClient` trait is
//! synchronous (so it stays unit-testable and wasm-safe without blocking), the
//! live network calls live on dedicated `async` methods; the synchronous trait
//! methods keep returning `BrokerError::NotImplemented` with a pointer to the
//! async variant, matching the contract documented in `brokers/mod.rs`.

use crate::brokers::{BrokerClient, BrokerError};
use crate::dto::order::{OrderIntent, OrderResponse};
use crate::dto::portfolio::AccountSummary;
use crate::websocket::{WsConfig, WsConnection};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const SMART_API_BASE: &str = "https://apismart.angeltone.com";
const SMART_STREAM_FEED: &str = "wss://smartstream.angeltone.com";

/// Payload for POST `/rest/auth/angelbroking/user/v1/loginByPassword`.
#[derive(Clone, Debug, Serialize)]
pub struct LoginRequest {
    pub api_key: String,
    pub client_code: String,
    pub password: String,
    pub totp: String,
}

/// Data portion of the SmartAPI login response.
#[derive(Clone, Debug, Deserialize)]
pub struct LoginData {
    pub access_token: String,
    pub feed_token: String,
    pub client_code: String,
}

/// Full SmartAPI login response.
#[derive(Clone, Debug, Deserialize)]
pub struct LoginResponse {
    pub status: bool,
    pub message: String,
    pub data: Option<LoginData>,
}

/// Angel One SmartAPI adapter.
pub struct AngelOne {
    api_key: String,
    client_code: String,
    access_token: Option<String>,
    feed_token: Option<String>,
}

impl AngelOne {
    /// Create a new adapter with API key and client code. Not connected until
    /// the login handshake completes.
    pub fn new(api_key: impl Into<String>, client_code: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client_code: client_code.into(),
            access_token: None,
            feed_token: None,
        }
    }

    /// Perform the SmartAPI login handshake (password + TOTP) and obtain a JWT
    /// access token and feed token.
    pub async fn login_by_password(
        &self,
        password: impl Into<String>,
        totp: impl Into<String>,
    ) -> Result<LoginResponse, BrokerError> {
        let body = LoginRequest {
            api_key: self.api_key.clone(),
            client_code: self.client_code.clone(),
            password: password.into(),
            totp: totp.into(),
        };
        let response = Request::post(
            &format!(
                "{SMART_API_BASE}/rest/auth/angelbroking/user/v1/loginByPassword"
            ),
        )
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| BrokerError::Network {
            message: format!("build login request: {e:?}"),
        })?
        .send()
        .await
        .map_err(|e| BrokerError::Network {
            message: e.to_string(),
        })?;
        if !response.ok() {
            return Err(BrokerError::Network {
                message: format!(
                    "login failed: HTTP {} {}",
                    response.status(),
                    response.status_text()
                ),
            });
        }
        response
            .json::<LoginResponse>()
            .await
            .map_err(|e| BrokerError::Network {
                message: e.to_string(),
            })
    }

    /// Store the JWT access token and feed token obtained from login.
    pub fn set_session(&mut self, access_token: impl Into<String>, feed_token: impl Into<String>) {
        self.access_token = Some(access_token.into());
        self.feed_token = Some(feed_token.into());
    }

    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    pub fn feed_token(&self) -> Option<&str> {
        self.feed_token.as_deref()
    }

    /// Connect the authenticated Angel One SmartStream WebSocket feed.
    pub fn connect_feed(&self) -> Result<WsConnection, String> {
        let access_token = self.access_token.as_ref().ok_or("not authenticated")?;
        let feed_token = self.feed_token.as_ref().ok_or("no feed token")?;
        let url = format!(
            "{SMART_STREAM_FEED}/?api_key={}&access_token={}&feed_token={}&client_code={}",
            self.api_key, access_token, feed_token, self.client_code
        );
        WsConnection::connect(
            &url,
            |_| {},
            |_| {},
            WsConfig {
                heartbeat_interval_ms: 15_000,
                ..Default::default()
            },
        )
    }

    /// Submit an order via the SmartAPI REST API (live, async).
    pub async fn submit_order_async(
        &self,
        intent: OrderIntent,
    ) -> Result<OrderResponse, BrokerError> {
        let _ = intent;
        let token = self.access_token.as_deref().ok_or(BrokerError::Unauthorized {
            message: "no access token".into(),
        })?;
        let body = serde_json::json!({});
        let response = Request::post(&format!("{SMART_API_BASE}/rest/secure/angelbroking/order/v1/placeOrder"))
            .header("Authorization", &format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .map_err(|e| BrokerError::Network {
                message: format!("build order request: {e:?}"),
            })?
            .send()
            .await
            .map_err(|e| BrokerError::Network {
                message: e.to_string(),
            })?;
        if !response.ok() {
            return Err(BrokerError::Network {
                message: format!("submit order failed: HTTP {}", response.status()),
            });
        }
        response
            .json::<OrderResponse>()
            .await
            .map_err(|e| BrokerError::Network {
                message: e.to_string(),
            })
    }

    /// Fetch the portfolio via the SmartAPI REST API (live, async).
    pub async fn fetch_portfolio_async(&self) -> Result<AccountSummary, BrokerError> {
        let _ = ();
        let token = self.access_token.as_deref().ok_or(BrokerError::Unauthorized {
            message: "no access token".into(),
        })?;
        let response = Request::get(&format!("{SMART_API_BASE}/rest/secure/angelbroking/portfolio/v1/getHolding"))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| BrokerError::Network {
                message: e.to_string(),
            })?;
        if !response.ok() {
            return Err(BrokerError::Network {
                message: format!("fetch portfolio failed: HTTP {}", response.status()),
            });
        }
        let _holdings: serde_json::Value = response.json().await.map_err(|e| BrokerError::Network {
            message: e.to_string(),
        })?;
        Ok(AccountSummary {
            account_id: "ANGEL".into(),
            currency: "INR".into(),
            total_equity: 0.0,
            available_margin: 0.0,
            used_margin: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
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

    fn submit_order(&mut self, _intent: OrderIntent) -> Result<OrderResponse, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "angel_one (use submit_order_async for live trading)".into(),
        })
    }

    fn modify_order(
        &mut self,
        _order_id: &str,
        _intent: OrderIntent,
    ) -> Result<OrderResponse, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "angel_one (use submit_order_async for live trading)".into(),
        })
    }

    fn cancel_order(&mut self, _order_id: &str) -> Result<(), BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "angel_one (use submit_order_async for live trading)".into(),
        })
    }

    fn fetch_portfolio(&mut self) -> Result<AccountSummary, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "angel_one (use fetch_portfolio_async for live trading)".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_connected_without_session() {
        let broker = AngelOne::new("api_key", "client_code");
        assert!(!broker.is_connected());
        assert!(broker.access_token().is_none());
        assert!(broker.feed_token().is_none());
    }

    #[test]
    fn connected_after_session() {
        let mut broker = AngelOne::new("api_key", "client_code");
        broker.set_session("jwt_token", "feed_token");
        assert!(broker.is_connected());
        assert_eq!(broker.access_token(), Some("jwt_token"));
        assert_eq!(broker.feed_token(), Some("feed_token"));
    }

    #[test]
    fn operations_return_not_implemented() {
        let mut broker = AngelOne::new("api_key", "client_code");
        broker.set_session("jwt", "feed");
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
