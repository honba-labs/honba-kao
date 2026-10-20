//! Workbench order-draft slice — the order ticket form (symbol, side, type,
//! product/margin, quantity, prices, time in force), the ticket panel and
//! order modal visibility, and the LTP lookup that seeds the draft. Owned by
//! [`super::workbench_store::WorkbenchStore`].

use leptos::prelude::*;

use crate::data::demo_data;
use crate::data::symbol_search;

// ---------------------------------------------------------------------------
// Order ticket domain
// ---------------------------------------------------------------------------

/// The symbol every fresh pane, ticket and demo position starts on.
pub(super) fn default_symbol() -> &'static str {
    "RELIANCE"
}

/// A single editable field on the order ticket (the `setTicketField` union).
#[derive(Clone, Debug, PartialEq)]
pub enum TicketField {
    Symbol(String),
    Side(OrderSide),
    OrderType(OrderType),
    Product(Product),
    Quantity(i64),
    Price(f64),
    TriggerPrice(f64),
    TimeInForce(TimeInForce),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum OrderSide {
    #[default]
    Buy,
    Sell,
}

impl OrderSide {
    pub const fn is_buy(self) -> bool {
        matches!(self, OrderSide::Buy)
    }

    pub const fn label(self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }

    pub const fn as_id(self) -> &'static str {
        match self {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        }
    }

    /// Position side wording used in the drawer (`LONG` / `SHORT`).
    pub const fn position_label(self) -> &'static str {
        match self {
            OrderSide::Buy => "LONG",
            OrderSide::Sell => "SHORT",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum OrderType {
    #[default]
    Limit,
    Market,
    StopMarket,
    StopLimit,
}

impl OrderType {
    /// `market | limit | stop_market | stop_limit` for the wire.
    pub const fn as_id(self) -> &'static str {
        match self {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::StopMarket => "stop_market",
            OrderType::StopLimit => "stop_limit",
        }
    }

    /// Short pill labels.
    pub const fn short(self) -> &'static str {
        match self {
            OrderType::Market => "MKT",
            OrderType::Limit => "LMT",
            OrderType::StopMarket => "SL-M",
            OrderType::StopLimit => "SL",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Product {
    #[default]
    Intraday,
    Delivery,
}

impl Product {
    pub const fn as_id(self) -> &'static str {
        match self {
            Product::Intraday => "intraday",
            Product::Delivery => "delivery",
        }
    }

    /// Ticket pill labels.
    pub const fn label(self) -> &'static str {
        match self {
            Product::Intraday => "Intraday (MIS 5x)",
            Product::Delivery => "Delivery (CNC)",
        }
    }

    /// Margin fraction: MIS needs 20%, CNC needs full notional.
    pub const fn margin_factor(self) -> f64 {
        match self {
            Product::Intraday => 0.2,
            Product::Delivery => 1.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TimeInForce {
    #[default]
    Day,
    Ioc,
}

impl TimeInForce {
    pub const fn as_id(self) -> &'static str {
        match self {
            TimeInForce::Day => "day",
            TimeInForce::Ioc => "ioc",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            TimeInForce::Day => "Day",
            TimeInForce::Ioc => "IOC",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OrderTicketDraft {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub product: Product,
    pub quantity: i64,
    pub price: f64,
    pub trigger_price: f64,
    pub tif: TimeInForce,
}

/// LTP used to seed the ticket and depth: the bundled universe (the search
/// pool carries `?tab`-free real prices), then demo quotes, then ₹1000.
pub fn instrument_price(symbol: &str) -> f64 {
    let upper = symbol.to_uppercase();
    if let Some(item) = symbol_search::metadata(&upper) {
        if item.price > 0.0 {
            return item.price;
        }
    }
    demo_data::demo_quotes()
        .iter()
        .find(|quote| quote.symbol.eq_ignore_ascii_case(symbol))
        .map_or(1000.0, |quote| quote.last)
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// The order-ticket draft and the two surfaces that edit it.
#[derive(Clone, Copy)]
pub struct OrderDraftStore {
    pub ticket: RwSignal<OrderTicketDraft>,
    pub ticket_open: RwSignal<bool>,
    pub modal_open: RwSignal<bool>,
}

impl OrderDraftStore {
    pub(super) fn new() -> Self {
        let ticket = OrderTicketDraft {
            symbol: default_symbol().to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            product: Product::Intraday,
            quantity: 25,
            price: instrument_price(default_symbol()),
            trigger_price: 0.0,
            tif: TimeInForce::Day,
        };

        Self {
            ticket: RwSignal::new(ticket),
            ticket_open: RwSignal::new(false),
            modal_open: RwSignal::new(false),
        }
    }

    pub fn set_ticket_field(&self, field: TicketField) {
        self.ticket.update(|ticket| match field {
            TicketField::Symbol(symbol) => ticket.symbol = symbol.to_uppercase(),
            TicketField::Side(side) => ticket.side = side,
            TicketField::OrderType(order_type) => ticket.order_type = order_type,
            TicketField::Product(product) => ticket.product = product,
            TicketField::Quantity(quantity) => ticket.quantity = quantity.max(1),
            TicketField::Price(price) => ticket.price = price.max(0.0),
            TicketField::TriggerPrice(trigger_price) => {
                ticket.trigger_price = trigger_price.max(0.0)
            }
            TicketField::TimeInForce(tif) => ticket.tif = tif,
        });
    }

    /// Quantity quick-increment from the stepper chips.
    pub fn shift_quantity(&self, delta: i64) {
        self.ticket.update(|ticket| {
            ticket.quantity = (ticket.quantity + delta).max(1);
        });
    }

    /// Point the ticket at a new symbol at its LTP (pane selection).
    pub fn sync_ticket_with_symbol(&self, symbol: &str) {
        let upper = symbol.to_uppercase();
        self.ticket.update(|ticket| {
            ticket.symbol = upper.clone();
            ticket.price = instrument_price(&upper);
        });
    }

    pub fn toggle_order_ticket(&self) {
        self.ticket_open.update(|open| *open = !*open);
    }

    pub fn open_order_ticket(&self, side: Option<OrderSide>, symbol: Option<&str>) {
        self.ticket_open.set(true);
        self.ticket.update(|ticket| {
            if let Some(side) = side {
                ticket.side = side;
            }
            if let Some(symbol) = symbol {
                let upper = symbol.to_uppercase();
                ticket.symbol = upper.clone();
                ticket.price = instrument_price(&upper);
            }
        });
    }

    pub fn close_order_ticket(&self) {
        self.ticket_open.set(false);
    }

    pub fn open_order_modal(
        &self,
        symbol: Option<&str>,
        side: Option<OrderSide>,
        price: Option<f64>,
        quantity: Option<i64>,
    ) {
        self.modal_open.set(true);
        self.ticket.update(|ticket| {
            let upper = symbol.map(|symbol| symbol.to_uppercase());
            let symbol = upper.as_ref().map_or(ticket.symbol.clone(), String::clone);
            ticket.symbol = symbol.clone();
            ticket.price = price.unwrap_or_else(|| instrument_price(&symbol));
            if let Some(side) = side {
                ticket.side = side;
            }
            if let Some(quantity) = quantity {
                ticket.quantity = quantity.max(1);
            }
        });
    }

    pub fn close_order_modal(&self) {
        self.modal_open.set(false);
    }
}
