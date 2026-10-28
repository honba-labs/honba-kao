//! Integration tests for `honba-kao-api-wasm::pairing`.
//!
//! Covers FIFO and LIFO fill-to-trip pairing and `TradeTrip` P&L extraction.

use honba_api_wasm::pairing::{pair_fills_fifo, pair_fills_lifo, Fill, TradeTrip};

fn fill(id: &str, symbol: &str, side: &str, price: f64, qty: f64, ts: i64) -> Fill {
    Fill { id: id.into(), symbol: symbol.into(), side: side.into(), price, quantity: qty, timestamp: ts }
}

#[test]
fn fifo_matches_buy_to_earliest_sell() {
    let fills = vec![
        fill("f1", "RELIANCE", "BUY", 100.0, 10.0, 1),
        fill("f2", "RELIANCE", "SELL", 110.0, 10.0, 2),
    ];
    let trips = pair_fills_fifo(&fills);
    assert_eq!(trips.len(), 1);
    assert!(trips[0].is_long);
    assert!((trips[0].pnl - 100.0).abs() < 1e-9); // (110-100)*10
}

#[test]
fn fifo_partial_fill_leaves_queue() {
    let fills = vec![
        fill("f1", "RELIANCE", "BUY", 100.0, 10.0, 1),
        fill("f2", "RELIANCE", "SELL", 110.0, 4.0, 2),
    ];
    let trips = pair_fills_fifo(&fills);
    assert_eq!(trips.len(), 1);
    assert!((trips[0].quantity - 4.0).abs() < 1e-9);
}

#[test]
fn lifo_matches_buy_to_latest_sell() {
    // Two buys, then a sell that matches the latest buy first (LIFO).
    let fills = vec![
        fill("f1", "RELIANCE", "BUY", 100.0, 5.0, 1),
        fill("f2", "RELIANCE", "BUY", 100.0, 5.0, 2),
        fill("f3", "RELIANCE", "SELL", 110.0, 10.0, 3),
    ];
    let trips = pair_fills_lifo(&fills);
    assert_eq!(trips.len(), 2);
    // Both trips are long (buy then sell), each 5 qty, pnl (110-100)*5 = 50
    for t in &trips {
        assert!(t.is_long);
        assert!((t.pnl - 50.0).abs() < 1e-9);
        assert!((t.quantity - 5.0).abs() < 1e-9);
    }
}

#[test]
fn pairing_empty() {
    assert!(pair_fills_fifo(&[]).is_empty());
    assert!(pair_fills_lifo(&[]).is_empty());
}

#[test]
fn trade_trip_fields() {
    let t = TradeTrip { symbol: "RELIANCE".into(), is_long: true, entry_price: 100.0, exit_price: 110.0, quantity: 5.0, pnl: 50.0, pnl_pct: 0.1, entry_time: 1, exit_time: 2, duration_seconds: 1 };
    assert_eq!(t.symbol, "RELIANCE");
    assert!(t.is_long);
    assert!((t.pnl - 50.0).abs() < 1e-9);
}
