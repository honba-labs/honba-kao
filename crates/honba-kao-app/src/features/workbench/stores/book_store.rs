//! Workbench book slice — the local, optimistic trading book (positions,
//! open orders, trade executions), the operator trading state, the status
//! banner, and the honba-ya backend wiring that resyncs them. Owned by
//! [`super::workbench_store::WorkbenchStore`].

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use honba_kao_client::wire::{
    InstrumentId, OrdersRequest, PortfolioPosition, WireExchange, WireOrder,
    WireOrderStatus, WireTradingState,
};
use rust_decimal::Decimal;

use super::order_draft_store::{
    OrderSide, OrderTicketDraft, OrderType, Product, TimeInForce, default_symbol, instrument_price,
};
use crate::shared::api::http::client;

// ---------------------------------------------------------------------------
// Book domain
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TradingState {
    #[default]
    Active,
    Reducing,
    Halted,
}

impl TradingState {
    pub const fn as_id(self) -> &'static str {
        match self {
            TradingState::Active => "active",
            TradingState::Reducing => "reducing",
            TradingState::Halted => "halted",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WbPosition {
    pub symbol: String,
    /// `Buy` = long, `Sell` = short.
    pub side: OrderSide,
    pub quantity: i64,
    pub avg_price: f64,
    pub ltp: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WbOrder {
    pub order_id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: i64,
    pub price: f64,
    pub status: String,
    pub product: Product,
    pub tif: TimeInForce,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WbTrade {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: i64,
    /// Brokerage/fees charged for the fill (0.03% of notional, like the labs
    /// mock book).
    pub costs: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Info(String),
    Error(String),
}

// ---------------------------------------------------------------------------
// Sequencing
// ---------------------------------------------------------------------------

/// Sequential component of demo book ids (`ORD-…-1`, `ORD-…-2`, …).
static ORDER_SEQ: AtomicU64 = AtomicU64::new(1);

/// A session-unique book id: `ORD-<unix-ms>-<n>`.
///
/// The same string rides the wire as `client_order_id`, ADR 0019's
/// idempotency key, so a double-clicked submit records exactly one row.
/// The wall-clock prefix is what makes it survive page reloads (a bare
/// counter would start again at 1 and replay an old ledger row as a no-op).
pub fn next_order_id() -> String {
    let seq = ORDER_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    let ms = js_sys::Date::now() as u64;
    format!("ORD-{ms}-{seq}")
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// The trading book, operator state and status banner.
#[derive(Clone, Copy)]
pub struct BookStore {
    pub positions: RwSignal<Vec<WbPosition>>,
    pub orders: RwSignal<Vec<WbOrder>>,
    pub trades: RwSignal<Vec<WbTrade>>,
    pub trading_state: RwSignal<TradingState>,
    pub status: RwSignal<Option<Status>>,
}

impl BookStore {
    pub(super) fn new() -> Self {
        let positions = vec![
            // Seeded like the labs mock book: RELIANCE + INFY longs.
            WbPosition {
                symbol: default_symbol().to_string(),
                side: OrderSide::Buy,
                quantity: 50,
                avg_price: 2920.5,
                ltp: 2954.2,
                realized_pnl: 1450.0,
                unrealized_pnl: (2954.2 - 2920.5) * 50.0,
            },
            WbPosition {
                symbol: "INFY".to_string(),
                side: OrderSide::Buy,
                quantity: 100,
                avg_price: 1880.0,
                ltp: 1895.4,
                realized_pnl: -320.0,
                unrealized_pnl: (1895.4 - 1880.0) * 100.0,
            },
        ];

        let orders = vec![
            WbOrder {
                order_id: "ORD-10927".to_string(),
                symbol: "HDFCBANK".to_string(),
                order_type: OrderType::Limit,
                side: OrderSide::Sell,
                quantity: 40,
                price: 1720.0,
                status: "accepted".to_string(),
                product: Product::Intraday,
                tif: TimeInForce::Day,
            },
            WbOrder {
                order_id: "ORD-10928".to_string(),
                symbol: "TCS".to_string(),
                order_type: OrderType::Limit,
                side: OrderSide::Buy,
                quantity: 20,
                price: 3950.0,
                status: "accepted".to_string(),
                product: Product::Intraday,
                tif: TimeInForce::Day,
            },
        ];

        Self {
            positions: RwSignal::new(positions),
            orders: RwSignal::new(orders),
            trades: RwSignal::new(Vec::new()),
            trading_state: RwSignal::new(TradingState::Active),
            status: RwSignal::new(None),
        }
    }

    // -- The local (optimistic) trading book ---------------------------------

    /// Submit the ticket: market orders fill into a trade + position; limit /
    /// stop orders land in the open-orders list as `accepted`. The UI updates
    /// immediately; a background `POST /orders` records the same request on
    /// the honba-ya ledger (`fetch_state` resyncs the book once it answers).
    pub fn place_order(&self, ticket: OrderTicketDraft) {
        if self.trading_state.get_untracked() == TradingState::Halted {
            self.status.set(Some(Status::Error(
                "Cannot place order: Trading state is HALTED".to_string(),
            )));
            return;
        }

        // Session-unique; doubles as the backend idempotency key (ADR 0019).
        let order_id = next_order_id();
        let price = if ticket.order_type == OrderType::Market {
            instrument_price(&ticket.symbol)
        } else {
            ticket.price
        };

        if ticket.order_type == OrderType::Market {
            let trade = WbTrade {
                order_id: order_id.clone(),
                symbol: ticket.symbol.clone(),
                side: ticket.side,
                price,
                quantity: ticket.quantity,
                costs: (price * ticket.quantity as f64 * 0.0003 * 100.0).round() / 100.0,
            };
            self.trades.update(|trades| trades.insert(0, trade));
            self.apply_fill(&ticket.symbol, ticket.side, ticket.quantity, price);
            self.status.set(Some(Status::Info(format!(
                "Market order {} filled for {}x {} @ ₹{:.2}",
                order_id, ticket.quantity, ticket.symbol, price
            ))));
        } else {
            let order = WbOrder {
                order_id: order_id.clone(),
                symbol: ticket.symbol.clone(),
                order_type: ticket.order_type,
                side: ticket.side,
                quantity: ticket.quantity,
                price,
                status: "accepted".to_string(),
                product: ticket.product,
                tif: ticket.tif,
            };
            self.orders.update(|orders| orders.insert(0, order));
            self.status.set(Some(Status::Info(format!(
                "Limit order {} placed for {}x {} @ ₹{:.2}",
                order_id, ticket.quantity, ticket.symbol, price
            ))));
        }

        // Record the same request on the ledger. The local intent (limit /
        // stop shapes the server does not parse today) is already on screen;
        // a backend rejection only drags an error line, like the labs app.
        let request = OrdersRequest {
            instrument_id: Some(InstrumentId::new(
                WireExchange::Nse,
                ticket.symbol.clone(),
            )),
            side: Some(ticket.side.as_id().to_string()),
            order_type: Some(ticket.order_type.as_id().to_string()),
            qty: Some(Decimal::from(ticket.quantity)),
            price: Some(
                rust_decimal::Decimal::from_f64_retain(price).unwrap_or_default(),
            ),
            tif: Some(ticket.tif.as_id().to_string()),
            client_order_id: Some(order_id.clone()),
        };
        let store = *self;
        spawn_local_scoped(async move {
            match client().post_orders(&request).await {
                Ok(_) => store.refresh_book_after(Duration::from_millis(600)),
                Err(error) => store.report_api_error("placing the order", error),
            }
        });
    }

    /// Buy/sell aggregate into the position book (average-price naive, like
    /// the labs optimistic mock).
    fn apply_fill(&self, symbol: &str, side: OrderSide, quantity: i64, price: f64) {
        self.positions.update(|positions| {
            let existing = positions
                .iter_mut()
                .find(|position| position.symbol == symbol);
            match existing {
                Some(position) => {
                    let new_qty = if side.is_buy() {
                        position.quantity + quantity
                    } else {
                        position.quantity - quantity
                    };
                    if new_qty <= 0 {
                        positions.retain(|position| position.symbol != symbol);
                    } else {
                        position.quantity = new_qty;
                        position.ltp = price;
                        position.unrealized_pnl = (price - position.avg_price) * new_qty as f64;
                    }
                }
                None => {
                    if side.is_buy() {
                        positions.push(WbPosition {
                            symbol: symbol.to_string(),
                            side: OrderSide::Buy,
                            quantity,
                            avg_price: price,
                            ltp: price,
                            realized_pnl: 0.0,
                            unrealized_pnl: 0.0,
                        });
                    }
                }
            }
        });
    }

    pub fn cancel_order(&self, order_id: &str) {
        self.orders.update(|orders| {
            orders.retain(|order| order.order_id != order_id);
        });
        self.status
            .set(Some(Status::Info(format!("Order {order_id} cancelled"))));

        // The route answers a plain ack; refresh the book so the cancelled
        // ledger row (the server marks it, it never deletes) shows up as
        // `cancelled` and the rest of history stays aligned.
        let store = *self;
        let id = order_id.to_string();
        spawn_local_scoped(async move {
            match client().cancel_order(&id).await {
                Ok(_) => store.refresh_book_after(Duration::from_millis(300)),
                Err(error) => store.report_api_error("cancelling the order", error),
            }
        });
    }

    pub fn close_position(&self, symbol: &str) {
        self.positions.update(|positions| {
            positions.retain(|position| position.symbol != symbol);
        });
        self.status
            .set(Some(Status::Info(format!("Closed position for {symbol}"))));
        self.request_backend_close();
    }

    pub fn close_all_positions(&self) {
        self.positions.set(Vec::new());
        self.status.set(Some(Status::Info(
            "All open positions have been closed".to_string(),
        )));
        self.request_backend_close();
    }

    /// Fire the bodyless `POST /positions/close` (the only close route the
    /// backend exposes — it squares off every ledger position at once) and
    /// resync the drawer once the reducing fills land.
    fn request_backend_close(&self) {
        let store = *self;
        spawn_local_scoped(async move {
            match client().close_all_positions().await {
                Ok(_) => store.refresh_book_after(Duration::from_millis(800)),
                Err(error) => store.report_api_error("closing positions", error),
            }
        });
    }

    /// Operator state change. `reason` is the operator-log text the backend
    /// records with the transition (`PUT /trading-state`). Optimistic like
    /// the rest of the book: the badge moves at once and rolls back if the
    /// backend refuses the transition.
    pub fn set_trading_state(&self, state: TradingState, reason: Option<&str>) {
        let previous = self.trading_state.get_untracked();
        self.trading_state.set(state);
        self.status.set(Some(Status::Info(format!(
            "Trading state transitioned to {}",
            state.as_id().to_uppercase()
        ))));

        let store = *self;
        let wire_state = to_wire_trading_state(state);
        let reason = reason.map(str::to_string);
        spawn_local_scoped(async move {
            match client()
                .put_trading_state(wire_state, reason.as_deref())
                .await
            {
                Ok(change) => store.trading_state.set(wire_trading_state(change.state)),
                Err(error) => {
                    store.trading_state.set(previous);
                    store.report_api_error("changing the trading state", error);
                }
            }
        });
    }

    /// Toolbar kill switch: halt ↔ resume.
    pub fn toggle_kill_switch(&self) {
        let next = match self.trading_state.get_untracked() {
            TradingState::Halted => TradingState::Active,
            _ => TradingState::Halted,
        };
        self.set_trading_state(
            next,
            Some(match next {
                TradingState::Active => "Resume from the toolbar kill switch",
                _ => "Toolbar kill switch toggle",
            }),
        );
    }

    // -- Backend wiring ------------------------------------------------------

    /// Pull the live book from the honba-ya backend — page mount time, and
    /// again whenever a write succeeds far enough to be worth resyncing.
    /// Each success overwrites the demo seeds for its slice; a slice that
    /// fails leaves its seeds in place (dev without a server) and logs one
    /// warning, so the workbench still demos offline.
    pub fn fetch_state(&self) {
        let store = *self;
        spawn_local_scoped(async move {
            match client().get_trading_state().await {
                Ok(state) => store.trading_state.set(wire_trading_state(state)),
                Err(error) => {
                    tracing::warn!("workbench: trading-state fetch failed: {error}")
                }
            }
            store.fetch_book();
        });
    }

    /// Orders + positions + the derived trade book (one round trip each).
    fn fetch_book(&self) {
        let store = *self;
        spawn_local_scoped(async move {
            match client().get_orders(None, None).await {
                Ok(orders) => {
                    let book = orders
                        .iter()
                        .map(|row| wire_order(row.clone()))
                        .collect::<Vec<_>>();
                    store.orders.set(book);

                    // The trade book comes from fills; orders alone carry no
                    // fill data, so the trade list is empty until a trades
                    // endpoint is wired (P7). Orders are still mapped to the
                    // book for display.
                    let trades: Vec<WbTrade> = Vec::new();
                    store.trades.set(trades);
                }
                Err(error) => tracing::warn!("workbench: orders fetch failed: {error}"),
            }

            match client().get_positions().await {
                Ok(positions) => store.positions.set(
                    positions
                        .iter()
                        .map(|position| wire_position(position.clone()))
                        .collect(),
                ),
                Err(error) => tracing::warn!("workbench: positions fetch failed: {error}"),
            }
        });
    }

    /// Recount the book shortly after a backend write (fills arrive a beat
    /// after `POST /orders` and `POST /positions/close` answer).
    fn refresh_book_after(&self, delay: Duration) {
        let store = *self;
        set_timeout(move || store.fetch_book(), delay);
    }

    /// Surface a failed backend write as the ticket-style error banner.
    fn report_api_error(&self, doing: &str, error: honba_kao_client::ClientError) {
        let hint = error
            .retryable()
            .then(|| " (transient — try again)")
            .unwrap_or("");
        self.status.set(Some(Status::Error(format!(
            "Error while {doing}: {error}{hint}"
        ))));
    }

    // -- Status ---------------------------------------------------------------

    pub fn dismiss_status(&self) {
        self.status.set(None);
    }
}

// ---------------------------------------------------------------------------
// Wire mapping — honba-ya rows into view models
// ---------------------------------------------------------------------------

fn wire_trading_state(state: WireTradingState) -> TradingState {
    match state {
        WireTradingState::Reducing => TradingState::Reducing,
        WireTradingState::Halted => TradingState::Halted,
        _ => TradingState::Active,
    }
}

fn to_wire_trading_state(state: TradingState) -> WireTradingState {
    match state {
        TradingState::Reducing => WireTradingState::Reducing,
        TradingState::Halted => WireTradingState::Halted,
        TradingState::Active => WireTradingState::Active,
    }
}

fn wire_order(row: WireOrder) -> WbOrder {
    let price = match row.order_type {
        honba_kao_client::wire::WireOrderType::Limit(p) => {
            p.to_string().parse::<f64>().unwrap_or(0.0)
        }
        _ => 0.0,
    };
    WbOrder {
        order_id: row.client_order_id.to_string(),
        symbol: row.instrument.symbol.to_uppercase(),
        order_type: wire_order_type(row.order_type),
        side: if row.side == honba_kao_client::wire::WireOrderSide::Sell {
            OrderSide::Sell
        } else {
            OrderSide::Buy
        },
        quantity: row
            .quantity
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0) as i64,
        price,
        status: row.tag.clone().unwrap_or_default(),
        product: Product::Intraday,
        tif: match row.tif {
            honba_kao_client::wire::WireTimeInForce::Ioc => TimeInForce::Ioc,
            _ => TimeInForce::Day,
        },
    }
}

fn wire_order_type(order_type: honba_kao_client::wire::WireOrderType) -> OrderType {
    match order_type {
        honba_kao_client::wire::WireOrderType::Market => OrderType::Market,
        honba_kao_client::wire::WireOrderType::Limit(_) => OrderType::Limit,
        honba_kao_client::wire::WireOrderType::StopLoss(_) => OrderType::StopMarket,
        honba_kao_client::wire::WireOrderType::StopLimit { .. } => OrderType::StopLimit,
    }
}

#[allow(dead_code)]
fn wire_status_text(status: WireOrderStatus) -> String {
    match status {
        WireOrderStatus::PendingNew => "pending_new",
        WireOrderStatus::New => "new",
        WireOrderStatus::PartiallyFilled => "partially_filled",
        WireOrderStatus::Filled => "filled",
        WireOrderStatus::Cancelled => "cancelled",
        WireOrderStatus::Rejected => "rejected",
        WireOrderStatus::Expired => "expired",
    }
    .to_string()
}

fn wire_position(position: PortfolioPosition) -> WbPosition {
    let qty = position
        .quantity
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);
    let side = if qty < 0.0 {
        OrderSide::Sell
    } else {
        OrderSide::Buy
    };
    let ltp = if qty.abs() > 1e-9 {
        position.market_value.to_string().parse::<f64>().unwrap_or(0.0) / qty.abs()
    } else {
        position.avg_price.to_string().parse::<f64>().unwrap_or(0.0)
    };
    WbPosition {
        symbol: position.instrument.symbol.to_uppercase(),
        side,
        quantity: qty.abs() as i64,
        avg_price: position
            .avg_price
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0),
        ltp,
        realized_pnl: 0.0,
        unrealized_pnl: position
            .unrealized_pnl
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0),
    }
}

impl Status {
    pub fn text(&self) -> &str {
        match self {
            Status::Info(text) | Status::Error(text) => text,
        }
    }

    pub fn as_class(&self) -> &'static str {
        match self {
            Status::Info(_) => "wb-status wb-status--info",
            Status::Error(_) => "wb-status wb-status--error",
        }
    }
}
