//! Live stream frame parsing and subscribe message.

use honba_kao_client::stream::{Quality, StreamFrame, parse_frame, subscribe_message};

#[test]
fn quote_frame_carries_decimal_strings_and_provenance() {
    let text = r#"{"type":"quote","symbol":"RELIANCE","ltp":"2982.50","change_pct":"1.4",
        "provenance":{"quality":"delayed","source":"kite","as_of":"2026-10-26T10:00:00+05:30","latency_ms":42}}"#;
    match parse_frame(text).expect("quote parses") {
        StreamFrame::Quote {
            symbol,
            ltp,
            provenance,
            ..
        } => {
            assert_eq!(symbol, "RELIANCE");
            assert_eq!(ltp, "2982.50");
            assert_eq!(provenance.quality, Quality::Delayed);
            assert_eq!(provenance.latency_ms, Some(42));
            assert_eq!(provenance.note, None);
        }
        other => panic!("expected quote, got {other:?}"),
    }
}

#[test]
fn trade_side_is_optional() {
    let text = r#"{"type":"trade","symbol":"INFY","price":"1652.75","quantity":"25","ts":"2026-10-26T10:00:01Z"}"#;
    match parse_frame(text).expect("trade parses") {
        StreamFrame::Trade { side, quantity, .. } => {
            assert_eq!(side, None);
            assert_eq!(quantity, "25");
        }
        other => panic!("expected trade, got {other:?}"),
    }
}

#[test]
fn depth_frame_parses_both_sides() {
    let text = r#"{"type":"depth","symbol":"TCS","bids":[{"price":"4011.9","quantity":"100"}],
        "asks":[{"price":"4012.1","quantity":"80"}],"provenance":{"quality":"live","source":"kite"}}"#;
    match parse_frame(text).expect("depth parses") {
        StreamFrame::Depth {
            bids,
            asks,
            provenance,
            ..
        } => {
            assert_eq!(bids.len(), 1);
            assert_eq!(asks[0].price, "4012.1");
            assert_eq!(provenance.quality, Quality::Live);
        }
        other => panic!("expected depth, got {other:?}"),
    }
}

#[test]
fn unknown_frame_type_is_rejected() {
    assert!(parse_frame(r#"{"type":"mystery"}"#).is_err());
    assert!(parse_frame("not json").is_err());
}

#[test]
fn heartbeat_parses() {
    assert!(matches!(
        parse_frame(r#"{"type":"heartbeat","ts":"2026-10-26T10:00:00Z"}"#),
        Ok(StreamFrame::Heartbeat { .. })
    ));
}

#[test]
fn subscribe_message_lists_symbols() {
    let msg = subscribe_message(&["RELIANCE".to_string(), "TCS".to_string()]);
    let value: serde_json::Value = serde_json::from_str(&msg).expect("valid json");
    assert_eq!(value["action"], "subscribe");
    assert_eq!(value["symbols"][1], "TCS");
}
