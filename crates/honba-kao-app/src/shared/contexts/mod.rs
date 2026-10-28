//! Shared Leptos contexts wiring the application to `honba-kao-client`.
//!
//! This module owns the single `HonbaClientContext` provided at the app root
//! (`app.rs`). It binds the headless SSE/WebSocket transports from
//! `honba-kao-client` into Leptos reactive signals so feature pages can consume
//! live data instead of mounting dead mock values.

use honba_kao_client::{
    dto::{
        order::{OrderIntent, OrderResponse},
        portfolio::{AccountSummary, BrokerConnection, Holding, Position},
        risk::LiquidationWarning,
    },
    http::{
        cancel_order, get_brokers, get_holdings, get_portfolio_snapshot, get_positions,
        submit_order,
    },
    stream::{connect_agent_stream, connect_orders_stream, connect_risk_stream},
};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::EventSource;
/// Application-wide client context.
///
/// Owns reactive signals fed by SSE streams (`orders`, `risk`) plus one-off
/// REST snapshots (`account`, `positions`, `brokers`), and exposes async client
/// actions (`submit_order`, `cancel_order`) backed by `honba-kao-client`.
///
/// The underlying `EventSource` handles are retained for the lifetime of the
/// context so the SSE connections stay open (dropping an `EventSource` closes
/// the connection).
#[derive(Clone)]
pub struct HonbaClientContext {
    pub oms_connected: RwSignal<bool>,
    pub feed_connected: RwSignal<bool>,
    pub latency_ms: RwSignal<Option<u64>>,
    pub account: RwSignal<Option<AccountSummary>>,
    pub positions: RwSignal<Vec<Position>>,
    pub holdings: RwSignal<Vec<Holding>>,
    pub brokers: RwSignal<Vec<BrokerConnection>>,
    pub orders: RwSignal<Vec<OrderResponse>>,
    pub liquidation_warnings: RwSignal<Vec<LiquidationWarning>>,
    pub submit_order: Callback<OrderIntent>,
    pub cancel_order: Callback<String>,
    #[allow(dead_code)]
    pub(crate) _order_event_source: Option<EventSource>,
    #[allow(dead_code)]
    pub(crate) _risk_event_source: Option<EventSource>,
    #[allow(dead_code)]
    pub(crate) _agent_event_source: Option<EventSource>,
}

impl HonbaClientContext {
    /// Build the context, bind SSE streams to reactive signals, and kick off
    /// the initial REST snapshot fetch.
    pub fn new() -> Self {
        let oms_connected = RwSignal::new(false);
        let feed_connected = RwSignal::new(false);
        let latency_ms = RwSignal::new(None);
        let account = RwSignal::new(None);
        let positions = RwSignal::new(vec![]);
        let holdings = RwSignal::new(vec![]);
        let brokers = RwSignal::new(vec![]);
        let orders = RwSignal::new(vec![]);
        let liquidation_warnings = RwSignal::new(vec![]);

        // ---- Client actions backed by honba-kao-client ---------------------
        let submit_order = Callback::new(move |intent: OrderIntent| {
            let intent = intent.clone();
            spawn_local(async move {
                if let Err(e) = submit_order(&intent).await {
                    tracing::error!("submit_order failed: {e}");
                }
            });
        });

        let cancel_order = Callback::new(move |order_id: String| {
            spawn_local(async move {
                if let Err(e) = cancel_order(&order_id).await {
                    tracing::error!("cancel_order failed: {e}");
                }
            });
        });

        // ---- SSE stream bindings (orders + risk + agent) -------------------
        let orders_signal = orders.clone();
        let order_event_source = connect_orders_stream("/api/v1/orders/stream")
            .ok()
            .map(|stream| super::streams::bind_sse_stream(stream, orders_signal));
        if order_event_source.is_some() {
            feed_connected.set(true);
        }

        let warnings_signal = liquidation_warnings.clone();
        let risk_event_source = connect_risk_stream("/api/v1/risk/stream")
            .ok()
            .map(|stream| super::streams::bind_sse_stream(stream, warnings_signal));

        let agent_signal = RwSignal::new(Vec::<honba_kao_client::stream::AgentStep>::new());
        let agent_event_source = connect_agent_stream("/api/v1/agent/stream")
            .ok()
            .map(|stream| super::streams::bind_sse_stream(stream, agent_signal));

        // ---- Initial REST snapshots ----------------------------------------
        {
            let account = account.clone();
            let positions = positions.clone();
            let holdings = holdings.clone();
            let brokers = brokers.clone();
            let oms = oms_connected.clone();
            let latency = latency_ms.clone();
            spawn_local(async move {
                let start = js_swap_now();
                let _ = get_portfolio_snapshot()
                    .await
                    .map(|snap| account.set(Some(snap)));
                let _ = get_positions().await.map(|p| positions.set(p));
                let _ = get_holdings().await.map(|h| holdings.set(h));
                let _ = get_brokers().await.map(|b| {
                    let connected = b.iter().any(|br| br.is_connected);
                    brokers.set(b);
                    oms.set(connected);
                });
                latency.set(Some(js_swap_now().saturating_sub(start)));
            });
        }

        Self {
            oms_connected,
            feed_connected,
            latency_ms,
            account,
            positions,
            holdings,
            brokers,
            orders,
            liquidation_warnings,
            submit_order,
            cancel_order,
            _order_event_source: order_event_source,
            _risk_event_source: risk_event_source,
            _agent_event_source: agent_event_source,
        }
    }
}

fn js_swap_now() -> u64 {
    js_sys::Date::now() as u64
}
