//! Conformance tests for [`honba_api_wasm::pairing`], the fill → round-trip pairing engine.
//!
//! Mirrors the honba-fe `core/simulator/trade-pairing.test.ts` fixtures (see the module doc on
//! `pairing.rs` for the pairing rules). Builds wire [`FillReport`]s directly since the real type
//! is a plain struct, not the fictional builder API an earlier draft of this suite assumed.

use honba_api_wasm::pairing::{
    Direction, FillSide, OpenPosition, RoundTripExitReason, RoundTripStatus, TradeMarkerKind,
    fills_from_trades, markers_from_pairing, pair_trades, trip_for_marker,
};
use honba_core::rust_decimal::Decimal;
use honba_core::uuid::Uuid;
use honba_core::{Currency, Exchange, FillReport as Trade, InstrumentId, Money, OrderSide, Price, Quantity};

const JAN1: i64 = 1_704_153_600; // 2024-01-02 00:00 UTC (fe's Date.UTC(2024, 0, 2))

fn ts(minute: i64) -> honba_core::chrono::DateTime<honba_core::chrono::Utc> {
    honba_core::chrono::DateTime::from_timestamp(JAN1 + minute * 60, 0).expect("valid timestamp")
}

fn decimal(value: f64) -> Decimal {
    Decimal::try_from(value).expect("finite test value")
}

fn fill(side: OrderSide, quantity: f64, price: f64, minute: i64, costs: f64, symbol: &str) -> Trade {
    Trade::new(
        format!("f-{symbol}-{minute}"),
        format!("o-{side:?}-{minute}"),
        Uuid::nil(),
        InstrumentId::new(Exchange::Nse, symbol),
        side,
        Price(decimal(price)),
        Quantity(decimal(quantity)),
        "algo",
        ts(minute),
    )
    .with_fee(Money::new(decimal(costs), Currency::Inr))
}

fn buy(quantity: f64, price: f64, minute: i64) -> Trade {
    fill(OrderSide::Buy, quantity, price, minute, 0.0, "TCS")
}

fn sell(quantity: f64, price: f64, minute: i64) -> Trade {
    fill(OrderSide::Sell, quantity, price, minute, 0.0, "TCS")
}

#[test]
fn orders_fills_by_ts_event_and_keeps_wire_order_on_ties() {
    let (fills, dropped) = fills_from_trades(&[sell(1.0, 110.0, 30), buy(1.0, 100.0, 10)]);
    assert_eq!(
        fills.iter().map(|f| (f.side, f.ts)).collect::<Vec<_>>(),
        vec![(FillSide::Buy, JAN1 + 600), (FillSide::Sell, JAN1 + 1800)]
    );
    assert_eq!(dropped, 0);
}

#[test]
fn counts_the_epoch_placeholder_as_dropped() {
    // fe's "drops fills it cannot pair" branch: side, size and price are
    // guaranteed by the typed wire, but a 0 placeholder ts_event can't be
    // used. Everything else in that vitest case is unrepresentable here.
    let zero_ts = Trade::new(
        "f-zero",
        "o-zero",
        Uuid::nil(),
        InstrumentId::new(Exchange::Nse, "TCS"),
        OrderSide::Buy,
        Price(decimal(100.0)),
        Quantity(decimal(1.0)),
        "algo",
        honba_core::chrono::DateTime::from_timestamp(0, 0).expect("valid timestamp"),
    );
    let (fills, dropped) = fills_from_trades(&[zero_ts, buy(1.0, 100.0, 20)]);
    assert_eq!(fills.len(), 1);
    assert_eq!(dropped, 1);
}

// --- pairTrades — round trips ----------------------------------------------

#[test]
fn pairs_a_simple_long_round_trip_and_derives_pnl_net_of_fill_costs() {
    let result = pair_trades(&[
        fill(OrderSide::Buy, 100.0, 100.0, 10, 10.0, "TCS"),
        fill(OrderSide::Sell, 100.0, 110.0, 60, 5.0, "TCS"),
    ]);
    assert_eq!(result.trips.len(), 1);
    let trip = &result.trips[0];
    assert_eq!(trip.direction, Direction::Long);
    assert_eq!(trip.status, RoundTripStatus::Closed);
    assert_eq!(trip.quantity, 100.0);
    assert_eq!(trip.open_quantity, 0.0);
    assert_eq!(trip.avg_entry, 100.0);
    assert_eq!(trip.avg_exit, Some(110.0));
    assert_eq!(trip.exit_reason, RoundTripExitReason::PositionClosed);
    assert_eq!(trip.entry_fills.len(), 1);
    assert_eq!(trip.exit_fills.len(), 1);
    assert_eq!(trip.fees, 15.0);
    assert!((trip.realized_pnl.unwrap() - (10.0 * 100.0 - 15.0)).abs() < 1e-6);
    assert!((trip.return_pct.unwrap() - 0.0985).abs() < 1e-6);
    assert!(result.open_positions.is_empty());
}

#[test]
fn aggregates_scale_in_fills_into_one_weighted_entry() {
    let result = pair_trades(&[
        buy(50.0, 100.0, 10),
        buy(50.0, 110.0, 20),
        sell(100.0, 120.0, 30),
    ]);
    assert_eq!(result.trips.len(), 1);
    let trip = &result.trips[0];
    assert_eq!(trip.quantity, 100.0);
    assert_eq!(trip.avg_entry, 105.0);
    assert_eq!(trip.avg_exit, Some(120.0));
    assert_eq!(trip.entry_fills.len(), 2);
    assert!((trip.realized_pnl.unwrap() - 1500.0).abs() < 1e-6);
}

#[test]
fn folds_scale_out_exits_into_one_trip_with_two_exit_legs() {
    let result = pair_trades(&[
        buy(100.0, 100.0, 10),
        sell(50.0, 110.0, 20),
        sell(50.0, 120.0, 30),
    ]);
    assert_eq!(result.trips.len(), 1);
    let trip = &result.trips[0];
    assert_eq!(trip.exit_fills.len(), 2);
    assert_eq!(trip.quantity, 100.0);
    assert_eq!(trip.avg_exit, Some(115.0));
    assert_eq!(trip.exit_ts, Some(JAN1 + 1800));
    assert!((trip.realized_pnl.unwrap() - 1500.0).abs() < 1e-6);
}

#[test]
fn leaves_a_partially_closed_position_open_charging_only_the_closed_share_of_entry_costs() {
    let result = pair_trades(&[
        fill(OrderSide::Buy, 100.0, 100.0, 10, 20.0, "TCS"),
        fill(OrderSide::Sell, 40.0, 110.0, 60, 4.0, "TCS"),
    ]);
    let trip = &result.trips[0];
    assert_eq!(trip.status, RoundTripStatus::Open);
    assert_eq!(trip.exit_reason, RoundTripExitReason::OpenAtEnd);
    assert_eq!(trip.open_quantity, 60.0);
    assert_eq!(trip.exit_fills.len(), 1);
    // 40 closed: (110-100) * 40 - (20 * 40/100 + 4)
    assert!((trip.realized_pnl.unwrap() - (400.0 - 12.0)).abs() < 1e-6);
    assert_eq!(
        result.open_positions,
        vec![OpenPosition {
            instrument: "TCS.NSE".into(),
            direction: Direction::Long,
            quantity: 60.0,
            avg_price: 100.0,
            since_ts: JAN1 + 600,
        }]
    );
}

#[test]
fn treats_a_reversal_through_flat_as_two_trips_the_second_derived_as_a_flip() {
    let result = pair_trades(&[buy(100.0, 100.0, 10), sell(150.0, 110.0, 20)]);
    assert_eq!(result.trips.len(), 2);
    assert_eq!(result.trips[0].direction, Direction::Long);
    assert_eq!(result.trips[0].status, RoundTripStatus::Closed);
    assert_eq!(result.trips[0].quantity, 100.0);
    assert_eq!(result.trips[0].exit_reason, RoundTripExitReason::Flipped);
    assert_eq!(result.trips[1].direction, Direction::Short);
    assert_eq!(result.trips[1].status, RoundTripStatus::Open);
    assert_eq!(result.trips[1].quantity, 50.0);
    assert_eq!(result.trips[1].avg_entry, 110.0);
    assert_eq!(result.trips[1].open_quantity, 50.0);
    assert_eq!(result.trips[1].realized_pnl, None);
    assert_eq!(result.open_positions.len(), 1);
}

#[test]
fn pairs_a_short_round_trip_with_the_short_sign_on_pnl() {
    let result = pair_trades(&[sell(100.0, 200.0, 10), buy(100.0, 180.0, 40)]);
    let trip = &result.trips[0];
    assert_eq!(trip.direction, Direction::Short);
    assert_eq!(trip.avg_entry, 200.0);
    assert_eq!(trip.avg_exit, Some(180.0));
    assert!((trip.realized_pnl.unwrap() - 2000.0).abs() < 1e-6);
}

#[test]
fn keeps_instruments_independent() {
    let result = pair_trades(&[
        fill(OrderSide::Buy, 10.0, 100.0, 10, 0.0, "TCS"),
        fill(OrderSide::Buy, 20.0, 50.0, 11, 0.0, "INFY"),
        fill(OrderSide::Sell, 10.0, 120.0, 40, 0.0, "TCS"),
    ]);
    assert_eq!(result.trips.len(), 2);
    let mut instruments: Vec<&str> = result.trips.iter().map(|trip| trip.instrument.as_str()).collect();
    instruments.sort_unstable();
    assert_eq!(instruments, ["INFY.NSE", "TCS.NSE"]);
    let tcs = result.trips.iter().find(|t| t.instrument == "TCS.NSE").unwrap();
    let infy = result.trips.iter().find(|t| t.instrument == "INFY.NSE").unwrap();
    assert_eq!(tcs.status, RoundTripStatus::Closed);
    assert_eq!(infy.status, RoundTripStatus::Open);
}

#[test]
fn numbers_trips_in_entry_order_and_reports_nothing_for_an_empty_journal() {
    let result = pair_trades(&[buy(1.0, 100.0, 30), buy(1.0, 100.0, 10), sell(1.0, 110.0, 15)]);
    let ids: Vec<&str> = result.trips.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["trip-1", "trip-2"]);
    assert!(pair_trades(&[]).trips.is_empty());
    assert_eq!(pair_trades(&[]).dropped, 0);
}

// --- markersFromPairing -----------------------------------------------------

#[test]
fn places_an_entry_marker_per_entry_leg_and_an_exit_marker_per_exit_leg() {
    let result = pair_trades(&[
        buy(100.0, 100.0, 10),
        sell(60.0, 110.0, 20),
        sell(40.0, 120.0, 30),
    ]);
    let markers = markers_from_pairing(&result);
    assert_eq!(
        markers.iter().map(|m| (m.kind, m.side, m.ts, m.quantity)).collect::<Vec<_>>(),
        vec![
            (TradeMarkerKind::Entry, FillSide::Buy, JAN1 + 600, 100.0),
            (TradeMarkerKind::Exit, FillSide::Sell, JAN1 + 1200, 60.0),
            (TradeMarkerKind::Exit, FillSide::Sell, JAN1 + 1800, 40.0),
        ]
    );
    assert!(markers.iter().all(|m| !m.derived));
}

#[test]
fn adds_one_clearly_derived_marker_for_a_position_still_open_at_the_run_end() {
    let open = pair_trades(&[buy(50.0, 100.0, 10)]);
    let markers = markers_from_pairing(&open);
    // The fill is still marked as an entry; the open tail is added on top
    // and is the only derived one.
    assert_eq!(
        markers.iter().map(|m| m.kind).collect::<Vec<_>>(),
        vec![TradeMarkerKind::Entry, TradeMarkerKind::Open]
    );
    assert!(!markers[0].derived);
    assert_eq!(markers[1].kind, TradeMarkerKind::Open);
    assert!(markers[1].derived);
    assert_eq!(markers[1].quantity, 50.0);
    assert_eq!(markers[1].price, 100.0);
    assert_eq!(markers[1].ts, JAN1 + 600);
}

#[test]
fn gives_every_marker_a_stable_key_its_round_trip_can_be_found_by() {
    let result = pair_trades(&[
        buy(100.0, 100.0, 10),
        sell(60.0, 110.0, 20),
        sell(40.0, 120.0, 30),
    ]);
    let markers = markers_from_pairing(&result);
    let mut keys: Vec<&str> = markers.iter().map(|m| m.key.as_str()).collect();
    keys.sort_unstable();
    let unique = keys.len();
    keys.dedup();
    assert_eq!(keys.len(), unique);
    assert_eq!(trip_for_marker(&result, Some(&markers[1].trip_id)).unwrap().id, "trip-1");
    assert!(trip_for_marker(&result, Some("trip-99")).is_none());
    assert!(trip_for_marker(&result, None).is_none());
}

#[test]
fn golden_journal_pairs_into_three_trips_like_the_fe_fixture() {
    // The analytics.test.ts fixture journal (two winners, one loser),
    // pausing here to pin pairing on it: +100−2, −50−2, +50−2.
    let trades = vec![
        fill(OrderSide::Buy, 10.0, 100.0, 10, 1.0, "TCS"),
        fill(OrderSide::Sell, 10.0, 110.0, 10 + 10 * 1440, 1.0, "TCS"),
        fill(OrderSide::Buy, 10.0, 100.0, 10 + 31 * 1440, 1.0, "TCS"),
        fill(OrderSide::Sell, 10.0, 95.0, 10 + 40 * 1440, 1.0, "TCS"),
        fill(OrderSide::Buy, 10.0, 100.0, 10 + 60 * 1440, 1.0, "TCS"),
        fill(OrderSide::Sell, 10.0, 105.0, 10 + 70 * 1440, 1.0, "TCS"),
    ];
    let result = pair_trades(&trades);
    let pnl: Vec<f64> = result.trips.iter().map(|t| t.realized_pnl.unwrap()).collect();
    assert_eq!(pnl.len(), 3);
    assert!((pnl[0] - 98.0).abs() < 1e-6);
    assert!((pnl[1] + 52.0).abs() < 1e-6);
    assert!((pnl[2] - 48.0).abs() < 1e-6);
}
