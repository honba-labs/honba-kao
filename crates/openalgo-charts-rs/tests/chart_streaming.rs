//! Benchmark-style verification for Milestone 5: 60 FPS under 100 ticks/sec
//! streaming updates.
//!
//! `StreamingCandleChart::push`/`update_last` are plain Rust `VecDeque`
//! operations that never touch the bound canvas element, and the
//! serialisation checks below replicate exactly the `serde_json::to_string`
//! calls `CanvasEngine::set_series`/`update_last` make internally (see
//! `src/canvas/engine.rs`, used by `StreamingCandleChart::commit`/
//! `commit_last` in `src/canvas/candles.rs`). None of that requires a live
//! DOM, so every check below runs under plain `#[cfg(test)]` with no
//! `wasm32` gate, and `cargo test -p openalgo-charts-rs` actually exercises
//! it (issue #32: the whole file used to be
//! `#![cfg(target_arch = "wasm32")]`-gated, so none of this ever ran under
//! plain `cargo test`).
//!
//! What genuinely needs a live browser -- actually mounting a chart via
//! `window.honbaCharts.mountStatic` and pushing the JSON payload across the
//! bridge (only possible once `CanvasEngine::mount` succeeds and `self.id`
//! is `Some`, see `src/canvas/bridge.rs`) -- is NOT exercised here. This
//! crate has no `wasm-bindgen-test`/headless-browser harness wired up (no
//! `wasm-bindgen-test` dependency, no karma/wasm-pack-test config), so
//! there is currently no way to run that path under any `cargo test`
//! invocation, host or `wasm32-unknown-unknown`. Rather than fabricate a
//! `#[cfg(target_arch = "wasm32")]` test for it that would never actually
//! run under `cargo test --target wasm32-unknown-unknown` without that
//! harness -- dead code that would only claim coverage that doesn't exist
//! -- we leave that path untested and call it out here. Wiring up a real
//! browser harness for it is tracked separately from #32.
//!
//! `std::time::Instant` is used throughout rather than
//! `web_sys::Performance::now()`: it works identically on the host (and
//! would continue to work under wasm32 if a browser test harness existed
//! later), and nothing exercised here needs browser-only timing.

use openalgo_charts::canvas::StreamingCandleChart;
use openalgo_charts::types::CandleData;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::time::{Duration, Instant};
use wasm_bindgen::{JsCast, JsValue};

fn candle(time: i64, close: Decimal) -> CandleData {
    CandleData {
        time,
        open: close - dec!(1.0),
        high: close + dec!(2.0),
        low: close - dec!(2.0),
        close,
        volume: Some(dec!(100.0)),
    }
}

/// A `web_sys::HtmlCanvasElement` that is never dereferenced as a real DOM
/// node. `StreamingCandleChart::push`/`update_last` never touch the bound
/// canvas at all, and `commit`/`commit_last` bail out via
/// `self.id.ok_or_else(..)` (see `CanvasEngine::set_series`/`update_last` in
/// `src/canvas/engine.rs`) before any FFI call would reach the element --
/// since this chart is never `mount()`-ed, `self.id` stays `None`. So a null
/// placeholder is safe to build a chart around on a non-wasm32 host build.
/// Mirrors the `fake_canvas` helper in `tests/canvas_engine_tests.rs`.
fn fake_canvas() -> web_sys::HtmlCanvasElement {
    JsValue::NULL.unchecked_into()
}

#[test]
fn candle_push_meets_100_tps_budget() {
    // 100 ticks/sec => 10 ms per tick budget; target < 1 ms per tick.
    // N is deliberately large (far beyond any realistic viewport) to prove
    // `push`'s O(1) bounded-buffer eviction holds up under sustained load.
    const N: usize = 20_000;
    let mut chart = StreamingCandleChart::new(fake_canvas(), N);

    let start = Instant::now();
    for i in 0..N {
        chart.push(candle(i as i64, dec!(100.0) + Decimal::from(i as i64) * dec!(0.01)));
    }
    let elapsed = start.elapsed();
    let per_tick = elapsed / N as u32;
    assert!(
        per_tick < Duration::from_millis(1),
        "candle push {per_tick:?}/tick exceeds 1 ms budget for 100 ticks/sec (10 ms/tick)"
    );
    assert_eq!(chart.len(), N);
}

#[test]
fn full_commit_serialisation_meets_60_fps_budget() {
    // Serialisation cost of committing a full history (the render path) --
    // exactly what `CanvasEngine::set_series` does inside
    // `StreamingCandleChart::commit` -- must stay well under the per-frame
    // budget for 60 FPS (~16 ms). Unlike the push-stress-test above, `N`
    // here is a realistic `max_candles` viewport size: a full re-serialise
    // is the initial-load/resync path, not a per-tick one (per-tick updates
    // go through the much cheaper `update_last`/`commit_last`, see
    // `update_last_is_cheaper_than_full_commit` below), so it only ever
    // needs to cover what's actually visible on screen, not an unbounded
    // history.
    const N: usize = 2_000;
    let mut chart = StreamingCandleChart::new(fake_canvas(), N);
    for i in 0..N {
        chart.push(candle(i as i64, dec!(100.0) + Decimal::from(i as i64) * dec!(0.01)));
    }

    let candles: Vec<CandleData> = chart.candles().iter().cloned().collect();
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
    // Streaming should revise only the last bar in place (`update_last`),
    // and the render path should serialise only that one candle
    // (`commit_last`'s `CanvasEngine::update_last`) rather than re-serialise
    // the whole history (`commit`'s `CanvasEngine::set_series`) on every
    // tick. `commit`/`commit_last` themselves short-circuit before touching
    // `serde_json` when the chart isn't mounted (see the module doc
    // comment), so this test exercises their actual internal serialisation
    // work directly -- precisely what `CanvasEngine::set_series`/
    // `update_last` do once mounted -- alongside the real `push`/
    // `update_last` buffer mutation.
    const N: usize = 5_000;
    const SAMPLES: u32 = 20;

    let mut chart = StreamingCandleChart::new(fake_canvas(), N);
    for i in 0..N {
        chart.push(candle(i as i64, dec!(100.0) + Decimal::from(i as i64) * dec!(0.01)));
    }

    // "Full commit" cost: re-serialise the entire buffer, as `commit` does.
    // Best-of-`SAMPLES` to filter out scheduler/allocator noise.
    let mut full_commit_cost = Duration::MAX;
    for _ in 0..SAMPLES {
        let full: Vec<CandleData> = chart.candles().iter().cloned().collect();
        let start = Instant::now();
        let _ = serde_json::to_string(&full).unwrap();
        full_commit_cost = full_commit_cost.min(start.elapsed());
    }

    // "Update last" cost: revise the buffer in place, then serialise only
    // the one revised candle, as `commit_last` does.
    let mut update_last_cost = Duration::MAX;
    for i in 0..SAMPLES {
        let revised = candle(N as i64 + i as i64, dec!(200.0));
        let start = Instant::now();
        chart.update_last(revised.clone());
        let _ = serde_json::to_string(&revised).unwrap();
        update_last_cost = update_last_cost.min(start.elapsed());
    }

    assert_eq!(chart.len(), N); // revision never grows the bounded buffer
    assert!(
        update_last_cost < Duration::from_millis(1),
        "last-tick revision {update_last_cost:?} too slow for 100 ticks/sec"
    );
    assert!(
        update_last_cost < full_commit_cost,
        "update_last ({update_last_cost:?}) should be cheaper than a full \
         commit ({full_commit_cost:?}) of {N} candles -- it isn't, so the \
         streaming path isn't actually avoiding the full re-serialisation \
         cost it's meant to"
    );
}
