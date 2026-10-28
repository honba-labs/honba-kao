use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ticker {
    pub symbol: String,
    pub ltp: Decimal,
    pub change_abs: Decimal,
    pub change_pct: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub volume_24h: Decimal,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bar {
    pub timestamp: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub symbol: String,
    pub bid_price: Decimal,
    pub bid_size: Decimal,
    pub ask_price: Decimal,
    pub ask_size: Decimal,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BookLevelDto {
    pub price: Decimal,
    pub size: Decimal,
    pub order_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Level2Book {
    pub symbol: String,
    pub bids: Vec<BookLevelDto>,
    pub asks: Vec<BookLevelDto>,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TradeTick {
    pub symbol: String,
    pub price: Decimal,
    pub size: Decimal,
    pub timestamp: i64,
    pub is_buyer_maker: bool,
}
