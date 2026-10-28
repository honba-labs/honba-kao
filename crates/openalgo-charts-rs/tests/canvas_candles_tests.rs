#![cfg(target_arch = "wasm32")]
//! DOM-backed tests for `StreamingCandleChart`; they need a real canvas, so
//! they only run on wasm32.

use openalgo_charts::canvas::StreamingCandleChart;
use openalgo_charts::types::CandleData;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use wasm_bindgen::JsCast;

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

// A real canvas can't be constructed headless here; the buffer logic is what
// we exercise.
fn canvas() -> web_sys::HtmlCanvasElement {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.create_element("canvas").ok())
        .and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("canvas element")
}

#[test]
fn push_evicts_oldest_when_full() {
    let mut chart = StreamingCandleChart::new(canvas(), 3);
    chart.push(candle(1, dec!(10.0)));
    chart.push(candle(2, dec!(11.0)));
    chart.push(candle(3, dec!(12.0)));
    assert_eq!(chart.len(), 3);
    chart.push(candle(4, dec!(13.0))); // evicts time 1
    assert_eq!(chart.len(), 3);
    assert_eq!(chart.candles()[0].time, 2);
    assert_eq!(chart.candles()[2].time, 4);
}

#[test]
fn update_last_revises_in_place() {
    let mut chart = StreamingCandleChart::new(canvas(), 10);
    chart.set_candles(vec![candle(1, dec!(10.0)), candle(2, dec!(11.0))]);
    chart.update_last(candle(2, dec!(11.5)));
    assert_eq!(chart.len(), 2);
    assert_eq!(chart.candles()[1].close, dec!(11.5));
    // Appends when empty.
    let mut empty = StreamingCandleChart::new(canvas(), 10);
    empty.update_last(candle(5, dec!(20.0)));
    assert_eq!(empty.len(), 1);
}
