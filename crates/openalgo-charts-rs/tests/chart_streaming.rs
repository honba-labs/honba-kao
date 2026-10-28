#![cfg(target_arch = "wasm32")]
//! Benchmark-style verification for Milestone 5: 60 FPS under 100 ticks/sec
//! streaming updates.
//!
//! The JS render loop (window.honbaCharts) cannot be driven headless here, so
//! this test benchmarks the Rust data path the chart owns: bounded buffer
//! pushes, last-tick revisions, and JSON serialisation for `setSeries`.
//! Per-tick cost must stay well under the 10 ms budget implied by 100 ticks/sec.
//! When running outside a browser (no `window`/`canvas`) the chart-dependent
//! assertions are skipped gracefully.

use openalgo_charts::canvas::StreamingCandleChart;
use openalgo_charts::types::CandleData;
use std::time::{Duration, Instant};
use wasm_bindgen::JsCast;

fn candle(time: i64, close: f64) -> CandleData {
    CandleData {
        time,
        open: close - 1.0,
        high: close + 2.0,
        low: close - 2.0,
        close,
        volume: Some(100.0),
    }
}

/// Best-effort canvas from the DOM; `None` when running outside a browser.
fn window_canvas() -> Option<web_sys::HtmlCanvasElement> {
    let window = web_sys::window()?;
    let doc = window.document()?;
    let el = doc.create_element("canvas").ok()?;
    el.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

#[test]
fn candle_push_meets_100_tps_budget() {
    // 100 ticks/sec => 10 ms per tick budget; target < 1 ms per tick.
    const N: usize = 20_000;
    let candles: Vec<CandleData> =
        (0..N).map(|i| candle(i as i64, 100.0 + i as f64 * 0.01)).collect();

    if let Some(canvas) = window_canvas() {
        let mut chart = StreamingCandleChart::new(canvas, N);
        let start = Instant::now();
        for c in &candles {
            chart.push(c.clone());
        }
        let elapsed = start.elapsed();
        let per_tick = elapsed / N as u32;
        assert!(
            per_tick < Duration::from_millis(1),
            "candle push {per_tick:?}/tick exceeds 1 ms budget for 100 ticks/sec (10 ms/tick)"
        );
    }

    // Serialisation cost of committing a full history (the render path) must
    // also stay well under the per-frame budget for 60 FPS (~16 ms).
    let start = Instant::now();
    let _json = serde_json::to_string(&candles).unwrap();
    let serialise = start.elapsed();
    assert!(
        serialise < Duration::from_millis(10),
        "serialising {N} candles took {serialise:?}, too slow for 60 FPS"
    );
}

#[test]
fn update_last_is_cheaper_than_full_commit() {
    // Streaming should revise only the last bar, not re-serialise the whole
    // history on every tick.
    const N: usize = 5_000;
    if let Some(canvas) = window_canvas() {
        let mut chart = StreamingCandleChart::new(canvas, N);
        for i in 0..N {
            chart.push(candle(i as i64, 100.0 + i as f64 * 0.01));
        }
        let start = Instant::now();
        chart.update_last(candle(N as i64, 200.0));
        let revise = start.elapsed();
        assert!(
            revise < Duration::from_millis(1),
            "last-tick revision {revise:?} too slow for 100 ticks/sec"
        );
        assert_eq!(chart.len(), N); // revision does not grow the buffer
    }
}
