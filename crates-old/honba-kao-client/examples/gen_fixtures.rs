//! Regenerates the golden decode fixtures in `tests/fixtures/` from the wire
//! crates themselves (HBD-2026-DES-005 P0).
//!
//! Run on the host: `cargo run -p honba-kao-client --example gen_fixtures` —
//! and commit the output. The committed JSON is the golden artifact the
//! integration test decodes; re-run after a schema bump and inspect the diff.
//!
//! The fixtures mirror what `honba-ya-cli serve` answers for the simulator
//! routes: bars for `TCS.NSE`, a completed and a failed backtest response, a
//! journal `TradesResponse`, and one `Status` + one `Record` `StreamFrame`.

use std::fs;
use std::path::Path;

use honba_api::wire::Trade;
use honba_api::{
    BacktestMetrics, BacktestResponse, BarsResponse, Event, Message, RunStatus, StreamFrame,
    TradesResponse,
};
use honba_core::chrono::{DateTime, Utc};
use honba_core::rust_decimal::Decimal;
use honba_core::uuid::Uuid;
use honba_core::{Bar, Exchange, InstrumentId, OrderSide, Price, Quantity};

fn ts(nanos: u64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp_nanos(nanos as i64)
}

fn price(rupees: &str) -> Price {
    Price::from_rupees(rupees).expect("fixture price parses")
}

fn qty(units: u64) -> Quantity {
    Quantity::from_u64(units)
}

/// The strategy IR fixture: a `sma` manifest compiled through the same
/// `honba-api::compile_strategy` the server runs, so the id and IR of the
/// committed golden file are exactly what `honba-ya-cli serve` would answer.
fn compiled_strategy() -> honba_api::CompiledStrategy {
    const MANIFEST_JSON: &str = r#"{
        "api_version": "1.0.0",
        "name": "sma",
        "source_hash": "sha256:abc",
        "universe": {"explicit": [{"symbol": "TCS", "exchange": "NSE"}]},
        "subscriptions": {"instruments": [{"symbol": "TCS", "exchange": "NSE"}]},
        "driving_timeframe": {"interval": 5, "aggregation": "minute"},
        "warmup_bars": 20
    }"#;
    let manifest = serde_json::from_str(MANIFEST_JSON).expect("fixture manifest parses");
    let mut catalog = honba_api::StrategyCatalog::default();
    honba_api::compile_strategy(&mut catalog, manifest).expect("fixture manifest verifies")
}

fn bar(instrument: &InstrumentId, open_rupees: &str, close_rupees: &str, ts_nanos: u64) -> Bar {
    let open = price(open_rupees);
    let close = price(close_rupees);
    Bar {
        instrument: instrument.clone(),
        open,
        high: Price::new(open.0 * Decimal::new(1002, 3)), // open * 1.002
        low: Price::new(open.0 * Decimal::new(999, 3)),   // open * 0.999
        close,
        volume: qty(12_500),
        timestamp: ts(ts_nanos),
    }
}

/// A fixed client order id so the golden fixture is deterministic.
fn fixed_uuid(seed: u64) -> Uuid {
    Uuid::from_u64_pair(0, seed)
}

fn trade(
    fill_id: &str,
    side: OrderSide,
    qty_units: u64,
    price_rupees: &str,
    ts_nanos: u64,
) -> Trade {
    Trade::new(
        fill_id,
        format!("BK-1-{ts_nanos}"),
        fixed_uuid(ts_nanos),
        InstrumentId::new(Exchange::Nse, "TCS"),
        side,
        price(price_rupees),
        qty(qty_units),
        "fixture-gen",
        ts(ts_nanos),
    )
}

fn main() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    fs::create_dir_all(&out).expect("fixture dir");

    let instrument = InstrumentId::new(Exchange::Nse, "TCS");
    let base = 1_700_000_705_000_000_000u64; // a 09:15 IST session open, 5m grid
    let bars: Vec<Bar> = (0..4)
        .map(|i| {
            let open = 22_000 + i * 25;
            let close = 22_010 + i * 25;
            bar(
                &instrument,
                &format!("{open}.00"),
                &format!("{close}.00"),
                base + i as u64 * 300_000_000_000,
            )
        })
        .collect();

    fn write_fixture(dir: &Path, name: &str, value: &impl serde::Serialize) {
        let json = serde_json::to_string_pretty(value).expect("serialize");
        fs::write(dir.join(name), json + "\n").expect("write fixture");
        println!("wrote {name}");
    }

    write_fixture(&out, "bars_response.json", &BarsResponse { bars });

    write_fixture(
        &out,
        "backtest_response_completed.json",
        &BacktestResponse {
            run_id: "2025-01-02-v1-0-0-b7a1c9de834f2a019b3c5d6e7f809a1b".to_string(),
            status: RunStatus::Completed,
            metrics: Some(BacktestMetrics {
                trades: 14,
                net_pnl: Decimal::new(4_523_055, 2), // 45230.55
                sharpe: Decimal::new(187, 2),        // 1.87
                max_drawdown: Decimal::new(621, 4),  // 0.0621
                total_return: Decimal::new(4523, 4), // 0.4523
            }),
            assumptions: Some(serde_json::json!({
                "not_modelled": ["market impact", "wire-in latency", "stamp duty"]
            })),
            validation: None,
            error: None,
        },
    );

    write_fixture(
        &out,
        "backtest_response_failed.json",
        &BacktestResponse {
            run_id: "2025-01-02-v1-0-0-b7a1c9de834f2a019b3c5d6e7f809a1b".to_string(),
            status: RunStatus::Failed,
            metrics: None,
            assumptions: None,
            validation: None,
            error: Some(honba_api::ErrorDetail::new(
                honba_api::ErrorCode::ValidationInvalidRequest,
                "unknown strategy \"not_compiled\"",
            )),
        },
    );

    write_fixture(
        &out,
        "strategies_response.json",
        &honba_api::StrategiesResponse {
            strategies: vec![compiled_strategy()],
        },
    );

    let t1 = trade(
        "F-1",
        OrderSide::Buy,
        10,
        "22010.00",
        base + 1_500_000_000_000,
    );
    let t2 = trade(
        "F-2",
        OrderSide::Sell,
        10,
        "22060.00",
        base + 6_000_000_000_000,
    );
    let t3 = trade(
        "F-3",
        OrderSide::Buy,
        5,
        "21980.00",
        base + 9_000_000_000_000,
    );
    write_fixture(
        &out,
        "trades_response.json",
        &TradesResponse {
            trades: vec![t1, t2.clone(), t3],
        },
    );

    let fill_event = Message::new(Event::Fill(t2), ts(base + 6_000_000_000_000));
    write_fixture(
        &out,
        "stream_frame_status.json",
        &StreamFrame::Status {
            status: RunStatus::Running,
            seq: 1,
            error: None,
        },
    );
    write_fixture(
        &out,
        "stream_frame_record.json",
        &StreamFrame::Record {
            seq: 1,
            message: fill_event,
        },
    );
}
