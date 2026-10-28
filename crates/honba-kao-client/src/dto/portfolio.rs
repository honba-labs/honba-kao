use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccountSummary {
    pub account_id: String,
    pub currency: String,
    pub total_equity: f64,
    pub available_margin: f64,
    pub used_margin: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Holding {
    pub symbol: String,
    pub quantity: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub side: String, // "LONG" or "SHORT"
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub leverage: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BrokerConnection {
    pub broker_id: String,
    pub name: String,
    pub is_connected: bool,
    pub latency_ms: Option<u64>,
}
