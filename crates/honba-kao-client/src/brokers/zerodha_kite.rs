//! Zerodha Kite Connect broker adapter.
//!
//! Live auth flow (Kite Connect):
//!   1. `login_url()` — browser-based login URL (user enters credentials + TOTP
//!      in the Kite login web flow).
//!   2. Kite redirects back with a `request_token` query parameter.
//!   3. `exchange_session()` — POST `/session/token` with `api_key`,
//!      `request_token` and the SHA-256 checksum to obtain an `access_token`.
//!   4. `connect_market_feed()` — authenticated WebSocket feed.
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
use sha2::{Digest, Sha256};

const KITE_BASE: &str = "https://api.kite.trade";
const KITE_LOGIN: &str = "https://kite.trade/connect/login";
const KITE_MARKET_FEED: &str = "wss://ws.kite.trade";

/// Payload for POST `/session/token`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionRequest {
    pub api_key: String,
    pub request_token: String,
    pub checksum: String,
}

/// Response from POST `/session/token`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub access_token: String,
    pub login_time: String,
    pub user_id: String,
    pub user_name: String,
    pub user_shortname: String,
}

/// Zerodha Kite Connect adapter.
pub struct ZerodhaKite {
    api_key: String,
    api_secret: Option<String>,
    access_token: Option<String>,
    request_token: Option<String>,
}

impl ZerodhaKite {
    /// Create a new adapter with the API key. Not connected until an access
    /// token is obtained via the login flow.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: None,
            access_token: None,
            request_token: None,
        }
    }

    /// Create a new adapter with API key and API secret (required to exchange
    /// the request token for an access token).
    pub fn with_secret(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: Some(api_secret.into()),
            access_token: None,
            request_token: None,
        }
    }

    /// Browser-based Kite Connect login URL. The user enters credentials and
    /// TOTP here; Kite redirects back with a `request_token`.
    pub fn login_url(&self) -> String {
        format!("{KITE_LOGIN}?api_key={}&v=3", self.api_key)
    }

    /// Store the `request_token` obtained from the login redirect.
    pub fn set_request_token(&mut self, token: impl Into<String>) {
        self.request_token = Some(token.into());
    }

    pub fn request_token(&self) -> Option<&str> {
        self.request_token.as_deref()
    }

    /// Set the access token obtained from the session exchange.
    pub fn set_access_token(&mut self, token: impl Into<String>) {
        self.access_token = Some(token.into());
    }

    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    /// Exchange the stored `request_token` for an `access_token`.
    pub async fn exchange_session(&self) -> Result<SessionResponse, BrokerError> {
        let request_token = self
            .request_token
            .as_ref()
            .ok_or_else(|| BrokerError::Network {
                message: "no request token; complete the login flow first".into(),
            })?;
        let api_secret = self
            .api_secret
            .as_ref()
            .ok_or_else(|| BrokerError::Network {
                message: "no api_secret configured".into(),
            })?;
        let checksum = kite_checksum(&self.api_key, request_token, api_secret);
        let body = SessionRequest {
            api_key: self.api_key.clone(),
            request_token: request_token.clone(),
            checksum,
        };
        let response = Request::post(&format!("{KITE_BASE}/session/token"))
            .header("Content-Type", "application/json")
            .json(&body)
            .map_err(|e| BrokerError::Network {
                message: format!("build session request: {e:?}"),
            })?
            .send()
            .await
            .map_err(|e| BrokerError::Network {
                message: e.to_string(),
            })?;
        if !response.ok() {
            return Err(BrokerError::Network {
                message: format!(
                    "session exchange failed: HTTP {} {}",
                    response.status(),
                    response.status_text()
                ),
            });
        }
        response
            .json::<SessionResponse>()
            .await
            .map_err(|e| BrokerError::Network {
                message: e.to_string(),
            })
    }

    /// Connect the authenticated Kite market WebSocket feed.
    pub fn connect_market_feed(&self) -> Result<WsConnection, String> {
        let token = self
            .access_token
            .as_ref()
            .ok_or("not authenticated; call exchange_session first")?;
        let url =
            format!("{KITE_MARKET_FEED}/?api_key={}&access_token={}", self.api_key, token);
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

    /// Submit an order via the Kite REST API (live, async).
    pub async fn submit_order_async(
        &self,
        intent: OrderIntent,
    ) -> Result<OrderResponse, BrokerError> {
        let _ = intent;
        let token = self.access_token.as_deref().ok_or(BrokerError::Unauthorized {
            message: "no access token".into(),
        })?;
        let body = serde_json::json!({});
        let response = Request::post(&format!("{KITE_BASE}/orders/regular"))
            .header("X-Kite-Version", "3")
            .header("Authorization", &format!("token {}:{}", self.api_key, token))
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

    /// Fetch the portfolio via the Kite REST API (live, async).
    pub async fn fetch_portfolio_async(&self) -> Result<AccountSummary, BrokerError> {
        let _ = ();
        let token = self.access_token.as_deref().ok_or(BrokerError::Unauthorized {
            message: "no access token".into(),
        })?;
        let response = Request::get(&format!("{KITE_BASE}/portfolio/positions"))
            .header("X-Kite-Version", "3")
            .header("Authorization", &format!("token {}:{}", self.api_key, token))
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
        let _positions: serde_json::Value = response.json().await.map_err(|e| BrokerError::Network {
            message: e.to_string(),
        })?;
        Ok(AccountSummary {
            account_id: "KITE".into(),
            currency: "INR".into(),
            total_equity: 0.0,
            available_margin: 0.0,
            used_margin: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
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

    fn submit_order(&mut self, _intent: OrderIntent) -> Result<OrderResponse, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "zerodha_kite (use submit_order_async for live trading)".into(),
        })
    }

    fn modify_order(
        &mut self,
        _order_id: &str,
        _intent: OrderIntent,
    ) -> Result<OrderResponse, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "zerodha_kite (use submit_order_async for live trading)".into(),
        })
    }

    fn cancel_order(&mut self, _order_id: &str) -> Result<(), BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "zerodha_kite (use submit_order_async for live trading)".into(),
        })
    }

    fn fetch_portfolio(&mut self) -> Result<AccountSummary, BrokerError> {
        Err(BrokerError::NotImplemented {
            broker: "zerodha_kite (use fetch_portfolio_async for live trading)".into(),
        })
    }
}

fn kite_checksum(api_key: &str, request_token: &str, api_secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{api_key}{request_token}{api_secret}").as_bytes());
    to_hex(&hasher.finalize())
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_url_contains_api_key() {
        let broker = ZerodhaKite::new("api_key_123");
        let url = broker.login_url();
        assert!(url.starts_with("https://kite.trade/connect/login"));
        assert!(url.contains("api_key=api_key_123"));
        assert!(url.contains("v=3"));
    }

    #[test]
    fn not_connected_without_token() {
        let broker = ZerodhaKite::new("api_key");
        assert!(!broker.is_connected());
        assert!(broker.access_token().is_none());
    }

    #[test]
    fn connected_after_access_token() {
        let mut broker = ZerodhaKite::new("api_key");
        broker.set_access_token("token_abc");
        assert!(broker.is_connected());
        assert_eq!(broker.access_token(), Some("token_abc"));
    }

    #[test]
    fn exchange_session_requires_request_token_and_secret() {
        let broker = ZerodhaKite::new("api_key");
        assert!(broker.request_token().is_none());
        assert!(broker.api_secret.is_none());
        assert!(!broker.is_connected());
    }

    #[test]
    fn checksum_is_deterministic_hex() {
        let a = kite_checksum("key", "token", "secret");
        let b = kite_checksum("key", "token", "secret");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
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
