//! Real-time ingestion regression gate (plan.md §7 DoD #5: "100+ WebSocket
//! ticks/sec and sub-second chart updates").
//!
//! Times the pure ingestion path a live socket frame takes before it reaches a
//! Leptos signal: `parse_frame` (JSON → `StreamFrame`) then
//! `QuoteBook::apply_frame`. Thresholds are deliberately conservative (100× the
//! DoD rate, and a 1 ms average budget against a 1 s requirement) so they hold
//! on a slow or loaded CI box in a debug build, while still catching an
//! accidental O(n²) or blocking regression. Fine-grained numbers live in the
//! criterion bench `benches/realtime_feed.rs`.

use std::time::{Duration, Instant};

use honba_kao_app::shared::command::symbols::MOCK_SYMBOLS;
use honba_kao_app::shared::stream::feed::QuoteBook;
use honba_kao_client::stream::parse_frame;

/// DoD #5 requirement: 100+ ticks/sec.
const DOD_TICKS_PER_SEC: u128 = 100;
/// Enforced floor: 100× the requirement.
const MIN_TICKS_PER_SEC: u128 = DOD_TICKS_PER_SEC * 100;
/// Enforced per-frame average latency ceiling (DoD budget is sub-second).
const MAX_AVG_LATENCY: Duration = Duration::from_millis(1);

const SYMBOL_COUNT: usize = 24;
const BURST_FRAMES: usize = 5_000;

/// The 11 mock tickers padded with synthetic ones to a realistic watchlist.
fn symbols() -> Vec<String> {
    let mut out: Vec<String> = MOCK_SYMBOLS.iter().map(|s| s.ticker.to_string()).collect();
    let mut n = 0;
    while out.len() < SYMBOL_COUNT {
        n += 1;
        out.push(format!("SYN{n:02}"));
    }
    out
}

/// A raw socket text frame. `ltp` is unique per index, so every frame is a
/// genuine state change rather than a dedup no-op.
fn quote_json(symbol: &str, i: usize) -> String {
    format!(
        r#"{{"type":"quote","symbol":"{symbol}","ltp":"{}.{:02}","change_pct":"0.5","provenance":{{"quality":"live","source":"kite"}}}}"#,
        1_000 + i / 100,
        i % 100
    )
}

fn burst() -> Vec<String> {
    let symbols = symbols();
    (0..BURST_FRAMES)
        .map(|i| quote_json(&symbols[i % symbols.len()], i))
        .collect()
}

#[test]
fn tick_burst_ingestion_far_exceeds_dod_throughput_and_latency() {
    let frames = burst();
    let mut book = QuoteBook::new();
    book.seed_synthetic(MOCK_SYMBOLS);

    let start = Instant::now();
    let mut changed = 0usize;
    for text in &frames {
        let frame = parse_frame(text).expect("well-formed quote frame");
        if book.apply_frame(&frame) {
            changed += 1;
        }
    }
    let elapsed = start.elapsed().max(Duration::from_nanos(1));

    assert_eq!(
        changed, BURST_FRAMES,
        "every frame must be a real state change"
    );

    let ticks_per_sec = (BURST_FRAMES as u128 * 1_000_000_000) / elapsed.as_nanos();
    let avg_latency = elapsed / BURST_FRAMES as u32;
    println!(
        "realtime_performance: {BURST_FRAMES} frames / {SYMBOL_COUNT} symbols in {elapsed:?} \
         → {ticks_per_sec} ticks/sec, avg {avg_latency:?}/frame"
    );

    assert!(
        ticks_per_sec >= MIN_TICKS_PER_SEC,
        "ingestion throughput {ticks_per_sec} ticks/sec below floor {MIN_TICKS_PER_SEC} \
         (DoD requires {DOD_TICKS_PER_SEC})"
    );
    assert!(
        avg_latency < MAX_AVG_LATENCY,
        "average per-frame latency {avg_latency:?} exceeds {MAX_AVG_LATENCY:?}"
    );
}
