//! Shared NSH context wiring the application to `honba-kao-client`.
//!
//! This module owns the single `HonbaClientContext` provided at the app root
//! (`app.rs`). It binds the headless SSE/WebSocket transports from
//! `honba-kao-client` into NSH reactive signals so feature pages can consume
//! live data instead of mounting dead mock values.
//!
//! Stage 3 of the gallery-rs port (Phase A.1 "decouple platform layer from
//! web-sys"): all transport work is split behind
//! `#[cfg(target_arch = "wasm32")]` over a single seam —
//! [`start_live_transport`] — so the **web runtime** keeps today's exact
//! behavior (SSE streams + REST snapshots) while the **native desktop
//! runtime** starts no transport at all and runs the shell offline: the
//! signals are born empty/disconnected, the actions log and no-op, and only
//! [`super::streams::StreamHandle`] (inert on native) crosses the boundary.
//! A native transport (a `tokio`-backed substitute for `gloo-net` /
//! `EventSource`) is a later milestone; the seam is the only code to swap.
//!
//! NSH port (nsh-port.md Phase 5): `leptos::prelude::RwSignal` became
//! `nsh_core::prelude::Signal` (`Copy`, same `.get()`/`.set()`/`.update()`
//! surface); `wasm_bindgen_futures::spawn_local` became
//! `nsh_core::prelude::spawn`, which must run under an `Owner` reactive
//! scope — `HonbaClientContext::new()` is therefore only ever called from
//! inside `App`'s own `#[component]` body (`app.rs`), never at module scope.
//! `leptos::prelude::Callback<T>` (a plain Rust-closure wrapper) has no NSH
//! equivalent — `nsh_core::element::Callback<A, R>` is a cross-language
//! (Rust↔JS) call handle, not a closure box, and can't be built from a bare
//! `move |x| { .. }` the way Leptos's could. `submit_order`/`cancel_order`
//! are therefore plain `Rc<dyn Fn(..)>` closures instead: this struct is
//! only ever read via `use_context`, never passed as a component prop, so
//! the `Props: Clone + PartialEq` bound that forces every *prop* callback
//! in this port into a `#[derive(Event)]` enum does not apply here.

use std::rc::Rc;

use honba_kao_client::dto::{
    market::Level2Book,
    order::{OrderIntent, OrderResponse},
    portfolio::{AccountSummary, BrokerConnection, Holding, Position},
    risk::LiquidationWarning,
};
use honba_kao_client::websocket::DepthFeed;
use nsh_core::prelude::{Signal, signal};

use super::streams::StreamHandle;

/// The cockpit's default depth instrument.
///
/// Canonical honba-core `InstrumentId` shape (`EXCHANGE:SYMBOL-TYPE`) and
/// the fixture symbol RELIANCE the cockpit's ladder is built around.
/// (Read only by the wasm32 transport — hence the native-side allow.)
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) const DEPTH_SYMBOL: &str = "NSE:RELIANCE-EQ";

/// The three retained stream handles (`orders`, `risk`, `agent`).
///
/// Web runtime: live `EventSource`s (dropped = closed). Native: all `None`
/// (the shell runs disconnected offline). Clonable because the context
/// derives `Clone` (on the web runtime `EventSource` is a cheap handle
/// clone). The handles are only *retained* — never read — so silence the
/// unused-field lint (mirrors the pre-A.1 retained fields).
#[derive(Clone, Default)]
#[allow(dead_code)]
struct TransportStart {
    orders: Option<StreamHandle>,
    risk: Option<StreamHandle>,
    agent: Option<StreamHandle>,
    /// Retained L2 depth socket handle (dropped = closed). `None` on the
    /// native twin / offline, where no depth transport starts.
    depth: Option<DepthFeed>,
}

/// Application-wide client context.
///
/// Owns reactive signals fed by SSE streams (`orders`, `risk`, `agent`)
/// plus one-off REST snapshots (`account`, `positions`, `brokers`), and
/// exposes async client actions (`submit_order`, `cancel_order`) backed
/// by `honba-kao-client`.
///
/// The underlying [`StreamHandle`]s are retained for the lifetime of the
/// context so the SSE connections stay open on the web runtime (dropping
/// an `EventSource` closes the connection).
#[derive(Clone)]
pub struct HonbaClientContext {
    pub oms_connected: Signal<bool>,
    pub feed_connected: Signal<bool>,
    pub latency_ms: Signal<Option<u64>>,
    pub account: Signal<Option<AccountSummary>>,
    pub positions: Signal<Vec<Position>>,
    pub depth: Signal<Option<Level2Book>>,
    pub holdings: Signal<Vec<Holding>>,
    pub brokers: Signal<Vec<BrokerConnection>>,
    pub orders: Signal<Vec<OrderResponse>>,
    pub liquidation_warnings: Signal<Vec<LiquidationWarning>>,
    /// Steps streamed from the analyst SSE feed (thoughts, tool calls,
    /// artifacts): the AI analyst console's live input.
    pub agent_thoughts: Signal<Vec<honba_kao_client::stream::AgentStep>>,
    pub submit_order: Rc<dyn Fn(OrderIntent)>,
    pub cancel_order: Rc<dyn Fn(String)>,
    #[allow(dead_code)]
    _transport: TransportStart,
}

impl HonbaClientContext {
    /// Build the context, bind SSE streams to reactive signals, and kick off
    /// the initial REST snapshot fetch. Must be called from inside a
    /// component body (needs the current `Owner` reactive scope that
    /// `signal`/`spawn`/the caller's own `provide_context` all rely on).
    ///
    /// Only the web runtime reaches the backend; the native desktop runtime
    /// runs the same signals offline (see the module docs).
    pub fn new() -> Self {
        let oms_connected = signal(false);
        let feed_connected = signal(false);
        let latency_ms = signal(None::<u64>);
        let account = signal(None::<AccountSummary>);
        let positions = signal(Vec::<Position>::new());
        let depth = signal(None::<Level2Book>);
        let holdings = signal(Vec::<Holding>::new());
        let brokers = signal(Vec::<BrokerConnection>::new());
        let orders = signal(Vec::<OrderResponse>::new());
        let liquidation_warnings = signal(Vec::<LiquidationWarning>::new());
        let agent_thoughts = signal(Vec::<honba_kao_client::stream::AgentStep>::new());

        // ---- Client actions backed by honba-kao-client ---------------------
        let submit_order: Rc<dyn Fn(OrderIntent)> = Rc::new(move |intent: OrderIntent| {
            #[cfg(target_arch = "wasm32")]
            {
                nsh_core::prelude::spawn(async move {
                    if let Err(e) = honba_kao_client::http::submit_order(&intent).await {
                        tracing::error!("submit_order failed: {e}");
                    }
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = &intent;
                tracing::warn!(
                    "submit_order requested on the native desktop runtime: no live transport yet (offline shell), dropping"
                );
            }
        });

        let cancel_order: Rc<dyn Fn(String)> = Rc::new(move |order_id: String| {
            #[cfg(target_arch = "wasm32")]
            {
                nsh_core::prelude::spawn(async move {
                    if let Err(e) = honba_kao_client::http::cancel_order(&order_id).await {
                        tracing::error!("cancel_order failed: {e}");
                    }
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = &order_id;
                tracing::warn!(
                    "cancel_order({order_id}) requested on the native desktop runtime: no live transport yet (offline shell), dropping"
                );
            }
        });

        // ---- Live transport (SSE streams + REST snapshots) -----------------
        let transport = start_live_transport(TransportContext {
            oms_connected,
            feed_connected,
            latency_ms,
            account,
            positions,
            depth,
            holdings,
            brokers,
            orders,
            liquidation_warnings,
            agent_thoughts,
        });

        Self {
            oms_connected,
            feed_connected,
            latency_ms,
            account,
            positions,
            depth,
            holdings,
            brokers,
            orders,
            liquidation_warnings,
            agent_thoughts,
            submit_order,
            cancel_order,
            _transport: transport,
        }
    }
}

impl Default for HonbaClientContext {
    fn default() -> Self {
        Self::new()
    }
}

impl HonbaClientContext {
    /// An offline mirror of the context: dead signals, no-op actions, no
    /// transport. Used where the shell renders without a provided context
    /// (a standalone footer preview) — the same shape a native run with no
    /// transport ends up in, minus the context hand-off.
    pub(crate) fn offline() -> Self {
        Self {
            oms_connected: signal(false),
            feed_connected: signal(false),
            latency_ms: signal(None),
            account: signal(None),
            positions: signal(vec![]),
            depth: signal(None),
            holdings: signal(vec![]),
            brokers: signal(vec![]),
            orders: signal(vec![]),
            liquidation_warnings: signal(vec![]),
            agent_thoughts: signal(vec![]),
            submit_order: Rc::new(|_: OrderIntent| {
                tracing::warn!("submit_order requested without a live context — dropped");
            }),
            cancel_order: Rc::new(|_: String| {
                tracing::warn!("cancel_order requested without a live context — dropped");
            }),
            _transport: TransportStart::default(),
        }
    }
}

/// The signals the live transport writes, passed as one bundle.
/// (On the native twin no field is read — that runtime starts no
/// transport — hence the allow.)
#[allow(dead_code)]
struct TransportContext {
    oms_connected: Signal<bool>,
    feed_connected: Signal<bool>,
    latency_ms: Signal<Option<u64>>,
    account: Signal<Option<AccountSummary>>,
    positions: Signal<Vec<Position>>,
    depth: Signal<Option<Level2Book>>,
    holdings: Signal<Vec<Holding>>,
    brokers: Signal<Vec<BrokerConnection>>,
    orders: Signal<Vec<OrderResponse>>,
    liquidation_warnings: Signal<Vec<LiquidationWarning>>,
    agent_thoughts: Signal<Vec<honba_kao_client::stream::AgentStep>>,
}

/// Starts the live backend transport. Web runtime: the three SSE streams
/// (`/api/v1/orders/stream`, `/api/v1/risk/stream`,
/// `/api/v1/agent/stream`) bound to their signals + the one-shot REST
/// snapshot fetch. Native: a no-op returning no handles — the shell runs
/// offline until a native transport lands.
#[cfg(target_arch = "wasm32")]
fn start_live_transport(ctx: TransportContext) -> TransportStart {
    use honba_kao_client::http::{
        get_brokers, get_holdings, get_portfolio_snapshot, get_positions,
    };
    use honba_kao_client::stream::{
        connect_agent_stream, connect_orders_stream, connect_risk_stream,
    };
    use honba_kao_client::websocket::ws_url;
    use nsh_core::prelude::spawn;

    let TransportContext {
        oms_connected,
        feed_connected,
        latency_ms,
        account,
        positions,
        depth,
        holdings,
        brokers,
        orders,
        liquidation_warnings,
        agent_thoughts,
    } = ctx;

    // ---- SSE stream bindings (orders + risk + agent) -------------------
    let orders_stream = connect_orders_stream("/api/v1/orders/stream")
        .ok()
        .map(|stream| super::streams::bind_sse_stream(stream, orders));
    if orders_stream.is_some() {
        feed_connected.set(true);
    }

    let risk_stream = connect_risk_stream("/api/v1/risk/stream")
        .ok()
        .map(|stream| super::streams::bind_sse_stream(stream, liquidation_warnings));

    let agent_stream = connect_agent_stream("/api/v1/agent/stream")
        .ok()
        .map(|stream| super::streams::bind_sse_stream(stream, agent_thoughts));

    // The L2 depth socket (§6 of the WS endpoints): subscribe the cockpit's
    // default instrument; the protocol://host comes from the browser page
    // (WebSocket needs absolute URLs).
    let depth_feed = match DepthFeed::subscribe(&ws_url("/ws/live"), DEPTH_SYMBOL, {
        move |book| depth.set(Some(book))
    }) {
        Ok(feed) => Some(feed),
        Err(err) => {
            tracing::warn!(
                "depth socket unavailable: {err} — the cockpit keeps its fixture ladder"
            );
            None
        }
    };

    // ---- Initial REST snapshots ----------------------------------------
    spawn(async move {
        let start = now_ms();
        let _ = get_portfolio_snapshot()
            .await
            .map(|snap| account.set(Some(snap)));
        let _ = get_positions().await.map(|p| positions.set(p));
        let _ = get_holdings().await.map(|h| holdings.set(h));
        let _ = get_brokers().await.map(|b| {
            let connected = b.iter().any(|br| br.is_connected);
            brokers.set(b);
            oms_connected.set(connected);
        });
        latency_ms.set(Some(now_ms().saturating_sub(start)));
    });

    TransportStart {
        orders: orders_stream,
        risk: risk_stream,
        agent: agent_stream,
        depth: depth_feed,
    }
}

/// Milliseconds on the web runtime's wall clock (snapshot latency).
#[cfg(target_arch = "wasm32")]
fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

/// Native twin: no transport, no timing — signals stay born-offline.
#[cfg(not(target_arch = "wasm32"))]
fn start_live_transport(_ctx: TransportContext) -> TransportStart {
    tracing::info!(
        "honba-kao-app native desktop runtime: no live transport (web-sys SSE/gloo-net are browser-only); the shell runs offline until a native transport lands"
    );
    TransportStart::default()
}
