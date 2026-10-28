//! Market feed / DTO tests.
//!
//! Verifies the `honba-kao-client` data-transfer objects serialize and
//! round-trip through JSON, and that the SSE stream constructor functions are
//! present. Pure-logic tests that run on the host.

use honba_kao_client::dto::market::{BookLevelDto, Level2Book, Ticker, TradeTick};
use honba_kao_client::dto::order::OrderResponse;
use honba_kao_client::dto::portfolio::AccountSummary;
use honba_kao_client::stream::{connect_agent_stream, connect_orders_stream, connect_risk_stream};
use serde_json;

fn sample_ticker() -> Ticker {
    Ticker {
        symbol: "RELIANCE".into(),
        ltp: rust_decimal::Decimal::new(294250, 2),
        change_abs: rust_decimal::Decimal::new(1255, 2),
        change_pct: rust_decimal::Decimal::new(52, 3),
        high_24h: rust_decimal::Decimal::new(295500, 2),
        low_24h: rust_decimal::Decimal::new(293800, 2),
        volume_24h: rust_decimal::Decimal::new(1520000000, 2),
        timestamp: 1_730_000_000,
    }
}

#[test]
fn ticker_serializes_and_roundtrips() {
    let ticker = sample_ticker();
    let json = serde_json::to_string(&ticker).expect("ticker serializes");
    let back: Ticker = serde_json::from_str(&json).expect("ticker deserializes");
    assert_eq!(back.symbol, "RELIANCE");
    assert_eq!(back.ltp, ticker.ltp);
    assert!(json.contains("RELIANCE"));
}

#[test]
fn account_summary_serializes_and_roundtrips() {
    let acct = AccountSummary {
        account_id: "ACC-001".into(),
        currency: "INR".into(),
        total_equity: rust_decimal::Decimal::new(50075000, 2),
        available_margin: rust_decimal::Decimal::new(37500000, 2),
        used_margin: rust_decimal::Decimal::new(12500000, 2),
        unrealized_pnl: rust_decimal::Decimal::new(850000, 2),
        realized_pnl: rust_decimal::Decimal::new(200000, 2),
    };
    let json = serde_json::to_string(&acct).expect("account serializes");
    let back: AccountSummary = serde_json::from_str(&json).expect("account deserializes");
    assert_eq!(back.account_id, "ACC-001");
    assert_eq!(back.total_equity, acct.total_equity);
}

#[test]
fn level2_book_serializes_and_roundtrips() {
    let book = Level2Book {
        symbol: "NIFTY50".into(),
        bids: vec![BookLevelDto {
            price: rust_decimal::Decimal::new(2449800, 2),
            size: rust_decimal::Decimal::new(15000, 2),
            order_count: 5,
        }],
        asks: vec![BookLevelDto {
            price: rust_decimal::Decimal::new(2450200, 2),
            size: rust_decimal::Decimal::new(12000, 2),
            order_count: 4,
        }],
        timestamp: 1_730_000_000,
    };
    let json = serde_json::to_string(&book).expect("book serializes");
    let back: Level2Book = serde_json::from_str(&json).expect("book deserializes");
    assert_eq!(back.symbol, "NIFTY50");
    assert_eq!(back.bids.len(), 1);
    assert_eq!(back.asks.len(), 1);
}

#[test]
fn order_response_serializes_and_roundtrips() {
    use honba_kao_client::dto::order::OrderStatus;
    let order = OrderResponse {
        order_id: "ORD-123".into(),
        symbol: "RELIANCE".into(),
        side: "BUY".into(),
        status: OrderStatus::Open,
        price: Some(rust_decimal::Decimal::new(294250, 2)),
        quantity: rust_decimal::Decimal::new(50000, 2),
        filled_quantity: rust_decimal::Decimal::new(0, 0),
        created_at: 1_730_000_000,
    };
    let json = serde_json::to_string(&order).expect("order serializes");
    let back: OrderResponse = serde_json::from_str(&json).expect("order deserializes");
    assert_eq!(back.order_id, "ORD-123");
    assert_eq!(back.status, OrderStatus::Open);
}

#[test]
fn trade_tick_serializes_and_roundtrips() {
    let tick = TradeTick {
        symbol: "INFY".into(),
        price: rust_decimal::Decimal::new(170050, 2),
        size: rust_decimal::Decimal::new(2500, 2),
        timestamp: 1_730_000_000,
        is_buyer_maker: false,
    };
    let json = serde_json::to_string(&tick).expect("tick serializes");
    let back: TradeTick = serde_json::from_str(&json).expect("tick deserializes");
    assert_eq!(back.symbol, "INFY");
    assert_eq!(back.is_buyer_maker, false);
}

#[test]
fn sse_stream_constructors_exist() {
    // These functions must be present (each takes a URL string and returns a
    // Result<SseStream<T>, String>). Referencing them proves they compile.
    let _f_orders: fn(
        &str,
    ) -> Result<
        honba_kao_client::stream::SseStream<honba_kao_client::dto::order::OrderResponse>,
        String,
    > = connect_orders_stream;
    let _f_risk: fn(
        &str,
    ) -> Result<
        honba_kao_client::stream::SseStream<honba_kao_client::dto::risk::LiquidationWarning>,
        String,
    > = connect_risk_stream;
    let _f_agent: fn(
        &str,
    ) -> Result<
        honba_kao_client::stream::SseStream<honba_kao_client::stream::AgentStep>,
        String,
    > = connect_agent_stream;
    let _ = (&_f_orders, &_f_risk, &_f_agent);
}
