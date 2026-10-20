//! Typed HTTP client for honba-ya-api.
//!
//! In dev, the base URL is "/api" and Trunk proxies to :8000.
//! In prod, set it to your API origin, e.g. "https://api.honba.in".
//!
//! Wire types come straight from `honba-kokoro-messages` (the envelope, the
//! order book) and `honba-ya-api` (the request/response DTOs) — the same
//! crates the server serializes with, so there is no client-side mirror to
//! drift. Every response is enveloped (`honba_messages::ResponseEnvelope`;
//! API `1.0.0`, schema `4`), and an unknown version axis is rejected here
//! rather than decoded with the wrong shape.
//! `POST /orders` uses; do not write bare-JSON decodes.
//!
//! Simulator surface (HBD-2026-DES-005 P0): `get_bars`, `get_strategies`,
//! `post_strategies`, `post_backtests`, `get_backtest` and
//! `get_backtest_journal` mirror the `honba-ya` runs/market/strategies routes,
//! all through the same enveloped decode path. The run's event tail is SSE
//! (`GET /backtests/{id}/events`); the transport choice is
//! **`web_sys::EventSource`** (see `events.rs`): the server streams
//! `StreamFrame`s as `data:` JSON without per-request auth headers, so the
//! browser-native `EventSource` keeps reconnect semantics for free and needs
//! no fetch-stream parsing. Application versions of the same decision live in
//! `events.rs`' module doc.
use gloo_net::http::{Request, RequestBuilder};
use honba_api::{
    BacktestRequest, BacktestResponse, BarsResponse, CompiledStrategy, DepthResponse,
    LayoutDocument, LayoutsResponse, OrdersRequest, OrdersResponse, PortfolioPositionsResponse,
    ProfileRevision, QuotesResponse, StrategiesResponse, SweepRequest, SweepResponse,
    TradesResponse, TradingStateChange, TradingStateRequest, TradingStateResponse,
};
use honba_entities::PortfolioPosition;
use honba_messages::{Order, QuoteTick, ResponseEnvelope, TradingState};
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Re-exports of the wire types this client speaks, so consumers can map
/// responses into view models without taking the schema crates themselves.
pub mod wire {
    pub use crate::StatusAck;
    pub use honba_analytics::ValidationBlock;
    pub use honba_api::{
        BacktestEventsQuery, BacktestMetrics, BacktestRequest, BacktestResponse, BarsQuery,
        BarsResponse, CompiledStrategy, LayoutDocument, LayoutsResponse, OrdersRequest,
        OrdersResponse, ProfileRevision, DEFAULT_BAR_SPEC, DEFAULT_INITIAL_CAPITAL, FILL_MODELS,
        MAX_COMMISSION_PER_FILL, MAX_SLIPPAGE_BPS, RunStatus, StreamFrame, StrategiesResponse,
        SweepRequest, SweepResponse, TradesResponse,
    };
    pub use honba_entities::{Currency, Money as WireMoney, PortfolioPosition, PositionSide, Trade};
    pub use honba_messages::Exchange;
    pub use honba_messages::{
        Bar, BarAggregation, BarSpecification, BarType, ErrorCode as WireErrorCode, ErrorCode,
        ErrorDetail, Event as WireEvent, Event, Exchange as WireExchange, InstrumentId,
        Message as WireMessage, Message, OrderId, PriceType, QuoteTick as WireQuoteTick,
        Order as WireOrder, OrderSide as WireOrderSide,
        OrderStatus as WireOrderStatus, OrderStatus, OrderType as WireOrderType, OrderType,
        Product as WireProduct, TimeInForce as WireTimeInForce,
        TradingState as WireTradingState, UnixNanos as WireUnixNanos,
    };
}

/// The simulator/event-stream surface (SSE), feature-gated: only the wasm app
/// opens an `EventSource`; host tests decode frames from fixtures instead.
#[cfg(target_arch = "wasm32")]
pub mod events;

/// Semantic version of the API surface the client decodes.
pub const API_VERSION: &str = honba_messages::API_VERSION;
/// Integer version of the wire shape the client decodes.
pub const SCHEMA_VERSION: u32 = honba_messages::SCHEMA_VERSION;

/// The `{"status": "..."}` ack body the write routes answer with
/// (`DELETE /orders/{id}`, `POST /positions/close`).
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct StatusAck {
    /// Short past-tense status word (`cancelled`, `closing`).
    pub status: String,
}

#[derive(Clone, Debug, Error)]
pub enum ClientError {
    #[error("network: {0}")]
    Network(String),

    #[error("decode: {0}")]
    Decode(String),

    #[error("status {0}")]
    Status(u16),

    /// The server answered with a failure envelope.
    #[error("api: {code}: {message}")]
    Api {
        /// The stable machine-readable code (snake_case).
        code: String,
        /// Human-readable description.
        message: String,
        /// Whether repeating the identical request is safe.
        retryable: bool,
    },
}

impl ClientError {
    /// Whether re-issuing the identical request can plausibly succeed.
    pub fn retryable(&self) -> bool {
        matches!(self, ClientError::Network(_) | ClientError::Status(_))
            || match self {
                ClientError::Api { retryable, .. } => *retryable,
                _ => false,
            }
    }
}

#[derive(Clone)]
pub struct ApiClient {
    pub base_url: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    // -- Transport -----------------------------------------------------------

    /// Plain JSON GET (no envelope unwrap) — legacy helper for namespaces
    /// that read non-honba endpoints.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let resp = Request::get(&format!("{}{}", self.base_url, path))
            .send()
            .await
            .map_err(|e| ClientError::Network(e.to_string()))?;
        if !resp.ok() {
            return Err(ClientError::Status(resp.status()));
        }
        resp.json::<T>()
            .await
            .map_err(|e| ClientError::Decode(e.to_string()))
    }

    /// Send one request and unwrap the response envelope into `T`.
    async fn send<T: DeserializeOwned>(
        &self,
        build: impl FnOnce(&str) -> RequestBuilder,
        path: &str,
        body: Option<String>,
    ) -> Result<T, ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let request = build(&url);
        let request = match body {
            Some(body) => request
                .header("content-type", "application/json")
                .body(body)
                .map_err(|e| ClientError::Network(e.to_string()))?,
            None => request
                .build()
                .map_err(|e| ClientError::Network(e.to_string()))?,
        };
        let resp = request
            .send()
            .await
            .map_err(|e| ClientError::Network(e.to_string()))?;

        if !resp.ok() && resp.status() >= 500 {
            return Err(ClientError::Status(resp.status()));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| ClientError::Network(e.to_string()))?;

        let envelope = serde_json::from_str::<ResponseEnvelope<T>>(&text)
            .map_err(|e| ClientError::Decode(format!("envelope: {e}")))?;

        check_versions(&envelope)?;

        envelope.into_result().map_err(|detail| ClientError::Api {
            code: crate::error_code_str(&detail.code),
            message: detail.message,
            retryable: detail.retryable,
        })
    }

    // -- Orders --------------------------------------------------------------

    /// `GET /orders` — the order book (optionally filtered).
    pub async fn get_orders(
        &self,
        exchange: Option<&str>,
        symbol: Option<&str>,
    ) -> Result<Vec<Order>, ClientError> {
        let mut path = "/orders".to_string();
        let mut first = true;
        for (key, value) in [("exchange", exchange), ("symbol", symbol)] {
            if let Some(value) = value {
                path.push(if first { '?' } else { '&' });
                path.push_str(key);
                path.push('=');
                path.push_str(&crate::encode_query(value));
                first = false;
            }
        }
        let response: OrdersResponse = self.send(Request::get, &path, None).await?;
        Ok(response.orders)
    }

    /// `POST /orders` — place an order; answers the recorded row(s).
    pub async fn post_orders(&self, request: &OrdersRequest) -> Result<Order, ClientError> {
        let body =
            serde_json::to_string(request).map_err(|e| ClientError::Decode(e.to_string()))?;
        let response: Order = self.send(Request::post, "/orders", Some(body)).await?;
        Ok(response)
    }

    /// `DELETE /orders/{id}` — cancel one order. The route answers a status
    /// ack (and is a no-op on terminal or unknown rows), never the full book.
    pub async fn cancel_order(&self, id: &str) -> Result<StatusAck, ClientError> {
        let path = format!("/orders/{}", crate::encode_query(id));
        self.send(Request::delete, &path, None).await
    }

    // -- Trading state -------------------------------------------------------

    /// `GET /trading-state` — the operator state write routes evaluate against.
    pub async fn get_trading_state(&self) -> Result<TradingState, ClientError> {
        let response: TradingStateResponse =
            self.send(Request::get, "/trading-state", None).await?;
        Ok(response.state)
    }

    /// `PUT /trading-state` — move the operator state; every transition is
    /// allowed and `reason` is logged with it.
    pub async fn put_trading_state(
        &self,
        state: TradingState,
        reason: Option<&str>,
    ) -> Result<TradingStateChange, ClientError> {
        let body = serde_json::to_string(&TradingStateRequest {
            state,
            reason: reason.map(str::to_string),
        })
        .map_err(|e| ClientError::Decode(e.to_string()))?;
        self.send(Request::put, "/trading-state", Some(body)).await
    }

    // -- Positions -----------------------------------------------------------

    /// `GET /portfolio/positions` — valued open positions across accounts.
    pub async fn get_positions(&self) -> Result<Vec<PortfolioPosition>, ClientError> {
        let response: PortfolioPositionsResponse = self
            .send(Request::get, "/portfolio/positions", None)
            .await?;
        Ok(response.positions)
    }

    /// `POST /positions/close` — closes ALL positions (the endpoint is
    /// bodyless and unscoped; ADR-consistent server behavior). It fires the
    /// reducing market orders and answers immediately with a status ack; the
    /// ledger settles a moment later.
    pub async fn close_all_positions(&self) -> Result<StatusAck, ClientError> {
        self.send(Request::post, "/positions/close", None).await
    }

    // -- Market data ---------------------------------------------------------

    /// `GET /quotes` — latest top-of-book for the requested symbols.
    pub async fn get_quotes(
        &self,
        symbols: &[&str],
        venue: Option<&str>,
    ) -> Result<Vec<QuoteTick>, ClientError> {
        let mut path = "/quotes".to_string();
        let mut first = true;
        if !symbols.is_empty() {
            path.push_str("?symbols=");
            path.push_str(&crate::encode_query(&symbols.join(",")));
            first = false;
        }
        if let Some(venue) = venue {
            path.push(if first { '?' } else { '&' });
            path.push_str("venue=");
            path.push_str(&crate::encode_query(venue));
        }
        let response: QuotesResponse = self.send(Request::get, &path, None).await?;
        Ok(response.quotes)
    }

    /// `GET /depth/{id}` — the order book for one instrument.
    pub async fn get_depth(
        &self,
        id: &str,
        levels: Option<u32>,
    ) -> Result<DepthResponse, ClientError> {
        let mut path = format!("/depth/{}", crate::encode_query(id));
        if let Some(levels) = levels {
            path.push_str(&format!("?depth={levels}"));
        }
        self.send(Request::get, &path, None).await
    }

    // -- Simulator (HBD-2026-DES-005 P0) -------------------------------------

    /// `GET /bars/{id}?tf=&from=&to=` — historical bars for one instrument,
    /// ascending `ts_event`. `from`/`to` are `YYYY-MM-DD` (or RFC3339); `tf`
    /// matches the server's `parse_timeframe` (`5m`, `1d`, …).
    pub async fn get_bars(
        &self,
        id: &str,
        tf: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<BarsResponse, ClientError> {
        let mut path = format!("/bars/{}", crate::encode_query(id));
        let mut first = true;
        for (key, value) in [("tf", tf), ("from", from), ("to", to)] {
            if let Some(value) = value {
                path.push(if first { '?' } else { '&' });
                path.push_str(key);
                path.push('=');
                path.push_str(&crate::encode_query(value));
                first = false;
            }
        }
        self.send(Request::get, &path, None).await
    }

    /// `GET /strategies` — every strategy compiled by this server process,
    /// ordered by content id.
    pub async fn get_strategies(&self) -> Result<StrategiesResponse, ClientError> {
        self.send(Request::get, "/strategies", None).await
    }

    /// `POST /strategies` — verify and compile one strategy manifest, keeping
    /// it under a content id. The manifest is passed as raw JSON (the graph
    /// shape authored in the strategy studio); source code is rejected by the
    /// server, so nothing else is ever sent.
    pub async fn post_strategies(
        &self,
        manifest: &serde_json::Value,
    ) -> Result<CompiledStrategy, ClientError> {
        let request = honba_api::StrategiesRequest {
            manifest: serde_json::from_value(manifest.clone())
                .map_err(|e| ClientError::Decode(format!("manifest: {e}")))?,
        };
        let body =
            serde_json::to_string(&request).map_err(|e| ClientError::Decode(e.to_string()))?;
        self.send(Request::post, "/strategies", Some(body)).await
    }

    /// `POST /backtests` — submit a run; the answer is the initial
    /// `BacktestResponse` (often `pending`, never awaited to completion here —
    /// the caller polls `get_backtest`).
    ///
    /// The request is serialized from the crate's own struct, so the server's
    /// `deny_unknown_fields` never sees a typo.
    pub async fn post_backtests(
        &self,
        request: &BacktestRequest,
    ) -> Result<BacktestResponse, ClientError> {
        let body =
            serde_json::to_string(request).map_err(|e| ClientError::Decode(e.to_string()))?;
        self.send(Request::post, "/backtests", Some(body)).await
    }

    /// `GET /backtests/{id}` — the run's current lifecycle state and, once
    /// `completed`, its metrics/validation block.
    pub async fn get_backtest(&self, id: &str) -> Result<BacktestResponse, ClientError> {
        let path = format!("/backtests/{}", crate::encode_query(id));
        self.send(Request::get, &path, None).await
    }

    /// `GET /backtests/{id}/journal` — the run's realized fills as a plain
    /// JSON `TradesResponse`. (The on-disk journal is NDJSON; the server maps
    /// it before answering.)
    pub async fn get_backtest_journal(&self, id: &str) -> Result<TradesResponse, ClientError> {
        let path = format!("/backtests/{}/journal", crate::encode_query(id));
        self.send(Request::get, &path, None).await
    }

    // -- Strategy studio (HBD-2026-DES-006 P0) --------------------------------

    /// `POST /strategies/verify` — the pure, idempotent verify step: compile
    /// only, no catalog entry. The answer is the normalized `StrategyIr`
    /// echo, typed server-side but read here as raw JSON (the studio only
    /// hands it back to the caller for display; adding the type would pull
    /// honba-strategy into the client for one read-only field).
    pub async fn post_strategies_verify(
        &self,
        manifest: &serde_json::Value,
    ) -> Result<serde_json::Value, ClientError> {
        let body =
            serde_json::to_string(manifest).map_err(|e| ClientError::Decode(e.to_string()))?;
        self.send(Request::post, "/strategies/verify", Some(body))
            .await
    }

    /// `POST /sweeps` — submit a parameter-sweep job. `SweepRequest::resolve`
    /// requires `strategy`, `params` (a `{name: [low, high, step]}` object)
    /// and non-zero `seed`; `trials` must equal the Cartesian grid size, or
    /// the server rejects with `trials_mismatched`.
    pub async fn post_sweeps(&self, request: &SweepRequest) -> Result<SweepResponse, ClientError> {
        let body =
            serde_json::to_string(request).map_err(|e| ClientError::Decode(e.to_string()))?;
        self.send(Request::post, "/sweeps", Some(body)).await
    }

    /// `GET /sweeps/{id}` — the sweep job's lifecycle state and, once
    /// `completed`, its ranked report.
    pub async fn get_sweeps_by_id(&self, id: &str) -> Result<SweepResponse, ClientError> {
        let path = format!("/sweeps/{}", crate::encode_query(id));
        self.send(Request::get, &path, None).await
    }

    /// `GET /profile/layouts` — every named server-side layout, by name.
    pub async fn get_profile_layouts(&self) -> Result<LayoutsResponse, ClientError> {
        self.send(Request::get, "/profile/layouts", None).await
    }

    /// `PUT /profile/layouts/{name}` — store one named layout document (an
    /// opaque JSON value server-side). The studio's library sync is
    /// best-effort over this: a failure keeps the strategy local.
    pub async fn put_profile_layout(
        &self,
        name: &str,
        document: &serde_json::Value,
    ) -> Result<ProfileRevision, ClientError> {
        let path = format!("/profile/layouts/{}", crate::encode_query(name));
        let body = serde_json::to_string(&LayoutDocument(document.clone()))
            .map_err(|e| ClientError::Decode(e.to_string()))?;
        self.send(Request::put, &path, Some(body)).await
    }
}

/// Reject an envelope that does not speak the client's wire shape.
fn check_versions<T>(envelope: &ResponseEnvelope<T>) -> Result<(), ClientError> {
    if envelope.api_version.as_str() != API_VERSION {
        return Err(ClientError::Decode(format!(
            "api version mismatch: client {API_VERSION}, server {}",
            envelope.api_version.as_str(),
        )));
    }
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(ClientError::Decode(format!(
            "schema version mismatch: client {SCHEMA_VERSION}, server {}",
            envelope.schema_version
        )));
    }
    Ok(())
}

/// Percent-encode a query/path value the way `encodeURIComponent` does.
///
/// Used for symbols and order ids — strict alphanumerics plus the few
/// characters instrument ids legitimately carry.
fn encode_query(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            otherwise => out.extend(format!("%{otherwise:02X}").chars()),
        }
    }
    out
}

/// The stable snake_case code of an [`honba_messages::ErrorCode`].
fn error_code_str(code: &honba_messages::ErrorCode) -> String {
    let Ok(json) = serde_json::to_string(code) else {
        return "unknown".to_string();
    };
    json.trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_query_values() {
        assert_eq!(encode_query("NIFTY50"), "NIFTY50");
        assert_eq!(encode_query("O-1"), "O-1");
        assert_eq!(encode_query("a b&c"), "a%20b%26c");
    }

    #[test]
    fn api_errors_are_not_marked_retryable_by_construction() {
        let detail = honba_messages::ErrorDetail::new(
            honba_messages::ErrorCode::OrderRejected,
            "no_position",
        );
        let error = ClientError::Api {
            code: crate::error_code_str(&detail.code),
            message: detail.message.clone(),
            retryable: detail.retryable,
        };
        assert_eq!(error.to_string(), "api: order_rejected: no_position");
        assert!(!error.retryable());
    }

    // -- Strategy studio conformance fixtures (HBD-2026-DES-006 P0) ----------

    /// The `schema/conformance/studio` JSON files, read relative to the crate.
    fn fixture(name: &str) -> String {
        for base in ["../..", "../../.."] {
            let path = format!(
                "{base}/schema/conformance/studio/{name}.json"
            );
            if std::fs::read_to_string(&path).is_ok() {
                return path;
            }
        }
        panic!("fixture `{name}` not found under schema/conformance/studio");
    }

    /// Decodes a fixture through the same envelope path `send()` uses,
    /// asserting the server can answer the studio's seams and the client
    /// can read it, without any shape drift.
    fn decode_fixture<T: serde::de::DeserializeOwned>(
        name: &str,
    ) -> Result<T, honba_messages::ErrorDetail> {
        let text = std::fs::read_to_string(fixture(name)).expect("fixture file");
        let envelope: honba_messages::ResponseEnvelope<T> =
            serde_json::from_str(&text).expect("fixture envelope decodes");
        envelope.into_result()
    }

    #[test]
    fn the_verify_response_fixture_decodes_to_the_ir_echo() {
        let ir: serde_json::Value =
            decode_fixture("strategies_verify_response").expect("verify echoes the IR");
        let manifest = ir
            .get("manifest")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str());
        assert_eq!(manifest, Some("sma_crossover"));
        // The IR normalizes the resolved timeframes list; the studio's
        // verify panel renders exactly these fields.
        assert_eq!(ir["warmup_bars"], 20);
        assert!(ir["timeframes"].is_array());
    }

    #[test]
    fn the_verify_error_fixture_carries_the_agent_readable_reason() {
        // StrategyManifest decode failure: verify answers 422 with
        // `context.reason = <IrError::code()>`, designed to be reactable
        // without parsing the human message (honba-ya verify.rs).
        let text = std::fs::read_to_string(fixture("verify_error_no_subscriptions"))
            .expect("fixture file");
        let envelope: honba_messages::ResponseEnvelope<serde_json::Value> =
            serde_json::from_str(&text).expect("fixture envelope decodes");
        let detail = envelope.into_result().expect_err("verify refused");
        let context = detail.context.expect("verify attaches context");
        assert_eq!(context["reason"].as_str(), Some("no_subscriptions"));
    }

    #[test]
    fn the_sweep_response_fixture_decodes() {
        let sweep: honba_api::SweepResponse =
            decode_fixture("sweep_response").expect("sweep report decodes");
        assert_eq!(sweep.status, honba_api::RunStatus::Completed);
        let report = sweep.report.expect("completed sweep carries a report");
        assert_eq!(report.ranked.len(), 2);
        assert!(report.best.is_some());
    }
}
