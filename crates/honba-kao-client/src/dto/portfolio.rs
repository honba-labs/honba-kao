use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountSummary {
    pub account_id: String,
    pub currency: String,
    pub total_equity: Decimal,
    pub available_margin: Decimal,
    pub used_margin: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Holding {
    pub symbol: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub current_price: Decimal,
    pub pnl: Decimal,
    pub pnl_pct: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub side: String, // "LONG" or "SHORT"
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub current_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub leverage: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BrokerConnection {
    pub broker_id: String,
    pub name: String,
    pub is_connected: bool,
    pub latency_ms: Option<u64>,
}
