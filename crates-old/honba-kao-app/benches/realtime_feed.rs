//! Real-time ingestion throughput (plan.md §7 DoD #5: 100+ ticks/sec).
//!
//! Benches the pure path a live socket frame takes before it reaches a Leptos
//! signal: `parse_frame` alone, and `parse_frame` + `QuoteBook::apply_frame`
//! across a multi-symbol watchlist. `Throughput::Elements` makes criterion
//! report frames/sec, directly comparable to the DoD target. The CI-enforced
//! floor lives in `tests/realtime_performance.rs`.
//!
//! Run: `cargo bench -p honba-kao-app --bench realtime_feed`

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use honba_kao_app::shared::command::symbols::MOCK_SYMBOLS;
use honba_kao_app::shared::stream::feed::QuoteBook;
use honba_kao_client::stream::parse_frame;

const SYMBOL_COUNT: usize = 24;
const BATCH: usize = 1_000;

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

/// A raw socket text frame; `ltp` is unique per index so no frame is a no-op.
fn quote_json(symbol: &str, i: usize) -> String {
    format!(
        r#"{{"type":"quote","symbol":"{symbol}","ltp":"{}.{:02}","change_pct":"0.5","provenance":{{"quality":"live","source":"kite","as_of":"2026-10-27T09:15:00Z","latency_ms":3}}}}"#,
        1_000 + i / 100,
        i % 100
    )
}

fn bench_parse_frame(c: &mut Criterion) {
    let text = quote_json("RELIANCE", 4_217);
    let mut group = c.benchmark_group("realtime_feed");
    group.throughput(Throughput::Elements(1));
    group.bench_function("parse_frame/quote", |b| {
        b.iter(|| parse_frame(black_box(&text)).unwrap())
    });
    group.finish();
}

fn bench_ingest_pipeline(c: &mut Criterion) {
    let symbols = symbols();

    let mut book = QuoteBook::new();
    book.seed_synthetic(MOCK_SYMBOLS);
    // Monotonically increasing across every call (never wraps back to an
    // `ltp` already in the book), so every frame in every iteration is a
    // genuine state change, not a dedup no-op against a prior iteration.
    let mut calls = 0u64;

    let mut group = c.benchmark_group("realtime_feed");
    group.throughput(Throughput::Elements(BATCH as u64));
    group.bench_function("parse_and_apply/24_symbols", |b| {
        b.iter(|| {
            let base = calls as usize * BATCH;
            calls += 1;
            let mut changed = 0usize;
            for i in 0..BATCH {
                let text = quote_json(&symbols[i % symbols.len()], base + i);
                let frame = parse_frame(black_box(&text)).unwrap();
                changed += usize::from(book.apply_frame(&frame));
            }
            black_box(changed)
        })
    });
    group.finish();
}

criterion_group!(benches, bench_parse_frame, bench_ingest_pipeline);
criterion_main!(benches);
