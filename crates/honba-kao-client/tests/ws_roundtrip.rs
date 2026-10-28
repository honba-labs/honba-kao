//! Mock WebSocket round-trip integration test suite.
//!
//! Validates the message contract that crosses the WebSocket/SSE boundary:
//! outgoing client frames (`OrderIntent`, subscription requests) and incoming
//! server frames (`EventEnvelope` carrying `TickerUpdate`, `OrderBookUpdate`,
//! `TradePrint`, `OrderStateChange`, `LiquidationAlert`).
//!
//! This test runs natively (no browser / `web_sys` required) and exercises the
//! exact `serde` serialization used by `honba-kao-client`'s WebSocket transport
//! (`websocket::connection.rs`) and domain events (`events.rs`). The wasm-side
//! `web_sys::WebSocket` wiring is exercised by the trunk/browser build.

use honba_kao_client::dto::market::{BookLevelDto, Level2Book, Quote, Ticker, TradeTick};
use honba_kao_client::dto::order::{Fill, OrderIntent, OrderResponse, OrderStatus};
use honba_kao_client::dto::portfolio::{AccountSummary, Holding, Position};
use honba_kao_client::dto::risk::{LiquidationWarning, MarginSummary};
use honba_kao_client::events::{DomainEvent, EventEnvelope};

fn round_trip<T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug>(
    value: &T,
    label: &str,
) {
    let json = serde_json::to_string(value).expect(&format!("serialize {label}"));
    let back: T = serde_json::from_str(&json).expect(&format!("deserialize {label}"));
    assert_eq!(*value, back, "round-trip mismatch for {label}: {json}");
}

#[test]
fn ticker_update_frame_round_trip() {
    let envelope = EventEnvelope {
        event_id: "evt-001".into(),
        timestamp: 1_730_000_000,
        payload: DomainEvent::TickerUpdate(Ticker {
            symbol: "RELIANCE".into(),
            ltp: 2500.50,
            change_abs: 12.5,
            change_pct: 0.50,
            high_24h: 2520.0,
            low_24h: 2480.0,
            volume_24h: 3_400_000.0,
            timestamp: 1_730_000_000,
        }),
    };
    round_trip(&envelope, "ticker update envelope");

    // Incoming WebSocket text frame deserializes into the typed envelope.
    let frame = r#"{"event_id":"evt-001","timestamp":1730000000,"payload":{"TickerUpdate":{"symbol":"RELIANCE","ltp":2500.5,"change_abs":12.5,"change_pct":0.5,"high_24h":2520.0,"low_24h":2480.0,"volume_24h":3400000.0,"timestamp":1730000000}}}"#;
    let parsed: EventEnvelope = serde_json::from_str(frame).expect("parse ticker frame");
    assert_eq!(parsed, envelope);
}

#[test]
fn order_book_update_frame_round_trip() {
    let envelope = EventEnvelope {
        event_id: "evt-002".into(),
        timestamp: 1_730_000_001,
        payload: DomainEvent::OrderBookUpdate(Level2Book {
            symbol: "NIFTY24NOV22500CE".into(),
            bids: vec![
                BookLevelDto { price: 2499.0, size: 1200.0, order_count: 15 },
                BookLevelDto { price: 2498.5, size: 800.0, order_count: 9 },
            ],
            asks: vec![
                BookLevelDto { price: 2500.5, size: 1100.0, order_count: 14 },
                BookLevelDto { price: 2501.0, size: 950.0, order_count: 11 },
            ],
            timestamp: 1_730_000_001,
        }),
    };
    round_trip(&envelope, "order book update envelope");
}

#[test]
fn trade_print_frame_round_trip() {
    let envelope = EventEnvelope {
        event_id: "evt-003".into(),
        timestamp: 1_730_000_002,
        payload: DomainEvent::TradePrint(TradeTick {
            symbol: "RELIANCE".into(),
            price: 2500.50,
            size: 50.0,
            timestamp: 1_730_000_002,
            is_buyer_maker: false,
        }),
    };
    round_trip(&envelope, "trade print envelope");
}

#[test]
fn order_state_change_frame_round_trip() {
    let envelope = EventEnvelope {
        event_id: "evt-004".into(),
        timestamp: 1_730_000_003,
        payload: DomainEvent::OrderStateChange(OrderResponse {
            order_id: "ORD-0001".into(),
            symbol: "RELIANCE".into(),
            side: "BUY".into(),
            status: OrderStatus::PartiallyFilled,
            price: Some(2500.50),
            quantity: 100.0,
            filled_quantity: 40.0,
            created_at: 1_730_000_003,
        }),
    };
    round_trip(&envelope, "order state change envelope");
}

#[test]
fn liquidation_alert_frame_round_trip() {
    let envelope = EventEnvelope {
        event_id: "evt-005".into(),
        timestamp: 1_730_000_004,
        payload: DomainEvent::LiquidationAlert(LiquidationWarning {
            symbol: "NIFTY24NOV22500PE".into(),
            position_size: 75.0,
            liquidation_price: 2450.0,
            mark_price: 2475.0,
            buffer_pct: 5.0,
            is_critical: true,
        }),
    };
    round_trip(&envelope, "liquidation alert envelope");
}

#[test]
fn outgoing_order_intent_frame_round_trip() {
    // Client -> server WebSocket frame: an OrderIntent serialized as text.
    let intent = OrderIntent {
        symbol: "RELIANCE".into(),
        side: "SELL".into(),
        order_type: "LIMIT".into(),
        price: Some(2510.0),
        trigger_price: Some(2505.0),
        quantity: 25.0,
    };
    round_trip(&intent, "order intent frame");
}

#[test]
fn fill_and_portfolio_dto_round_trip() {
    round_trip(
        &Fill {
            fill_id: "FIL-001".into(),
            order_id: "ORD-0001".into(),
            symbol: "RELIANCE".into(),
            side: "BUY".into(),
            price: 2500.50,
            quantity: 40.0,
            fee: 2.50,
            timestamp: 1_730_000_005,
        },
        "fill dto",
    );
    round_trip(
        &AccountSummary {
            account_id: "ACC-001".into(),
            currency: "INR".into(),
            total_equity: 4_850_000.0,
            available_margin: 2_810_000.0,
            used_margin: 1_760_000.0,
            unrealized_pnl: 42_500.0,
            realized_pnl: 12_000.0,
        },
        "account summary dto",
    );
    round_trip(
        &Holding {
            symbol: "TCS".into(),
            quantity: 500.0,
            average_price: 3160.0,
            current_price: 3240.0,
            pnl: 40_000.0,
            pnl_pct: 2.53,
        },
        "holding dto",
    );
    round_trip(
        &Position {
            symbol: "NIFTY24NOV22500CE".into(),
            side: "LONG".into(),
            quantity: 50.0,
            entry_price: 120.0,
            current_price: 145.0,
            unrealized_pnl: 1250.0,
            leverage: 5.0,
        },
        "position dto",
    );
    round_trip(
        &MarginSummary {
            total_collateral: 2_000_000.0,
            margin_utilized: 1_400_000.0,
            margin_available: 600_000.0,
            utilization_pct: 70.0,
            haircut_discount: 15.0,
        },
        "margin summary dto",
    );
    round_trip(
        &Quote {
            symbol: "RELIANCE".into(),
            bid_price: 2500.25,
            bid_size: 300.0,
            ask_price: 2500.75,
            ask_size: 420.0,
            timestamp: 1_730_000_000,
        },
        "quote dto",
    );
}

#[test]
fn mock_ws_connection_send_receive_contract() {
    // Simulates a WebSocket connection's send/receive contract without
    // requiring web_sys. Outgoing frames are OrderIntent JSON; incoming frames
    // are EventEnvelope JSON. Both directions must preserve the payload.
    struct MockWs {
        sent: Vec<String>,
        received: Vec<String>,
    }

    impl MockWs {
        fn new() -> Self {
            Self {
                sent: Vec::new(),
                received: Vec::new(),
            }
        }
        fn send(&mut self, frame: &str) {
            self.sent.push(frame.to_string());
        }
        fn receive(&mut self, frame: &str) {
            self.received.push(frame.to_string());
        }
    }

    let mut ws = MockWs::new();

    // Outgoing: client submits an order.
    let intent = OrderIntent {
        symbol: "INFY".into(),
        side: "BUY".into(),
        order_type: "MARKET".into(),
        price: None,
        trigger_price: None,
        quantity: 5.0,
    };
    let outgoing = serde_json::to_string(&intent).unwrap();
    ws.send(&outgoing);

    // Incoming: server acknowledges with an order state change.
    let incoming = EventEnvelope {
        event_id: "evt-ack".into(),
        timestamp: 1_730_000_010,
        payload: DomainEvent::OrderStateChange(OrderResponse {
            order_id: "ORD-ACK-1".into(),
            symbol: "INFY".into(),
            side: "BUY".into(),
            status: OrderStatus::Open,
            price: None,
            quantity: 5.0,
            filled_quantity: 0.0,
            created_at: 1_730_000_010,
        }),
    };
    let incoming_json = serde_json::to_string(&incoming).unwrap();
    ws.receive(&incoming_json);

    // Assert send/receive round-trip fidelity.
    assert_eq!(ws.sent.len(), 1);
    assert_eq!(ws.received.len(), 1);
    let sent_back: OrderIntent = serde_json::from_str(&ws.sent[0]).unwrap();
    assert_eq!(sent_back, intent);
    let received_back: EventEnvelope = serde_json::from_str(&ws.received[0]).unwrap();
    assert_eq!(received_back, incoming);
}
