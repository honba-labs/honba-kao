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

use honba_api::{
    BacktestMetrics, BacktestResponse, BarsResponse, RunStatus, StreamFrame,
    TradesResponse,
};
use honba_entities::{Currency, Money, Trade};
use honba_messages::{
    Bar, BarAggregation, BarSpecification, BarType, Event, Exchange, InstrumentId, Message,
    OrderId, OrderSide, PriceType, UnixNanos,
};

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

fn bar(instrument: &InstrumentId, open: f64, close: f64, ts: u64) -> Bar {
    let spec = BarSpecification::new(1, BarAggregation::Minute, PriceType::Last);
    Bar::new(
        BarType::new(instrument.clone(), spec),
        open,
        open * 1.002,
        open * 0.999,
        close,
        12_500.0,
        UnixNanos::from_u64(ts),
        UnixNanos::from_u64(ts),
    )
}

fn trade(side: OrderSide, qty: f64, price: f64, ts: u64) -> Trade {
    Trade::new(
        OrderId::new(format!("BK-1-{}", ts)),
        InstrumentId::new("TCS", Exchange::new("NSE")),
        side,
        qty,
        price,
        Currency::Inr,
        UnixNanos::from_u64(ts),
        UnixNanos::from_u64(ts),
    )
    .with_costs(Money::zero(Currency::Inr))
}

fn main() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    fs::create_dir_all(&out).expect("fixture dir");

    let instrument = InstrumentId::new("TCS", Exchange::new("NSE"));
    let base = 1_700_000_705_000_000_000u64; // a 09:15 IST session open, 5m grid
    let bars: Vec<Bar> = (0..4)
        .map(|i| bar(&instrument, 22_000.0 + i as f64 * 25.0, 22_010.0 + i as f64 * 25.0, base + i * 300_000_000_000))
        .collect();

    fn write_fixture(dir: &Path, name: &str, value: &impl serde::Serialize) {
        let json = serde_json::to_string_pretty(value).expect("serialize");
        fs::write(dir.join(name), json + "\n").expect("write fixture");
        println!("wrote {name}");
    }

    write_fixture(&out, "bars_response.json", &BarsResponse { bars });

    write_fixture(&out, 
        "backtest_response_completed.json",
        &BacktestResponse {
            run_id: "2025-01-02-v1-0-0-b7a1c9de834f2a019b3c5d6e7f809a1b".to_string(),
            status: RunStatus::Completed,
            metrics: Some(BacktestMetrics {
                trades: 14,
                net_pnl: 45_230.55,
                sharpe: 1.87,
                max_drawdown: 0.0621,
                total_return: 0.4523,
            }),
            assumptions: Some(serde_json::json!({
                "not_modelled": ["market impact", "wire-in latency", "stamp duty"]
            })),
            validation: None,
            error: None,
        },
    );

    write_fixture(&out, 
        "backtest_response_failed.json",
        &BacktestResponse {
            run_id: "2025-01-02-v1-0-0-b7a1c9de834f2a019b3c5d6e7f809a1b".to_string(),
            status: RunStatus::Failed,
            metrics: None,
            assumptions: None,
            validation: None,
            error: Some(honba_messages::ErrorDetail::new(
                honba_messages::ErrorCode::ValidationInvalidRequest,
                "unknown strategy \"not_compiled\"",
            )),
        },
    );

    write_fixture(&out, 
        "strategies_response.json",
        &honba_api::StrategiesResponse {
            strategies: vec![compiled_strategy()],
        },
    );

    write_fixture(&out, 
        "trades_response.json",
        &TradesResponse {
            trades: vec![
                trade(OrderSide::Buy, 10.0, 22_010.0, base + 1_500_000_000_000),
                trade(OrderSide::Sell, 10.0, 22_060.0, base + 6_000_000_000_000),
                trade(OrderSide::Buy, 5.0, 21_980.0, base + 9_000_000_000_000),
            ],
        },
    );

    let message = Message::new(
        Event::OrderFilled {
            order_id: OrderId::new("BK-1-2"),
            ts_event: UnixNanos::from_u64(base + 6_000_000_000_000),
            last_qty: 10.0,
            last_px: 22_060.0,
        },
        UnixNanos::from_u64(base + 6_000_000_000_000),
    );
    write_fixture(&out, 
        "stream_frame_status.json",
        &StreamFrame::Status {
            status: RunStatus::Running,
            seq: 1,
            error: None,
        },
    );
    write_fixture(&out, 
        "stream_frame_record.json",
        &StreamFrame::Record { seq: 1, message },
    );
}
