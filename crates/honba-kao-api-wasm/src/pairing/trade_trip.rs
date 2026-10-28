use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Fill {
    pub id: String,
    pub symbol: String,
    pub side: String, // "BUY" or "SELL"
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: i64,
}

impl Fill {
    pub fn new(id: &str, symbol: &str, side: &str, price: Decimal, quantity: Decimal, timestamp: i64) -> Self {
        Self {
            id: id.into(),
            symbol: symbol.into(),
            side: side.into(),
            price,
            quantity,
            timestamp,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TradeTrip {
    pub symbol: String,
    pub is_long: bool,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub quantity: Decimal,
    pub pnl: Decimal,
    pub pnl_pct: Decimal,
    pub entry_time: i64,
    pub exit_time: i64,
    pub duration_seconds: i64,
}
