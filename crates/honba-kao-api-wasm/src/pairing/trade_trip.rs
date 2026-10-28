use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Fill {
    pub id: String,
    pub symbol: String,
    pub side: String, // "BUY" or "SELL"
    pub price: f64,
    pub quantity: f64,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TradeTrip {
    pub symbol: String,
    pub is_long: bool,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub entry_time: i64,
    pub exit_time: i64,
    pub duration_seconds: i64,
}
