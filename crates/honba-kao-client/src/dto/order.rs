use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OrderIntent {
    pub symbol: String,
    pub side: String,       // "BUY" or "SELL"
    pub order_type: String, // "LIMIT", "MARKET", "SL", "SL-M"
    pub price: Option<Decimal>,
    pub trigger_price: Option<Decimal>,
    pub quantity: Decimal,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OrderResponse {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub status: OrderStatus,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub created_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Fill {
    pub fill_id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub fee: Decimal,
    pub timestamp: i64,
}
