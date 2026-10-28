use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ticker {
    pub symbol: String,
    pub ltp: f64,
    pub change_abs: f64,
    pub change_pct: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub volume_24h: f64,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub symbol: String,
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BookLevelDto {
    pub price: f64,
    pub size: f64,
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
    pub price: f64,
    pub size: f64,
    pub timestamp: i64,
    pub is_buyer_maker: bool,
}
