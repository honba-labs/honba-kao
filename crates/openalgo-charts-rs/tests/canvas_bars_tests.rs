#![cfg(target_arch = "wasm32")]
//! DOM-backed tests for `StreamingBarChart`; they need a real canvas, so they
//! only run on wasm32.

use openalgo_charts::canvas::StreamingBarChart;
use openalgo_charts::types::CandleData;
use wasm_bindgen::JsCast;

fn bar(time: i64, close: f64) -> CandleData {
    CandleData {
        time,
        open: close - 0.5,
        high: close + 1.0,
        low: close - 1.0,
        close,
        volume: Some(50.0),
    }
}

fn canvas() -> web_sys::HtmlCanvasElement {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.create_element("canvas").ok())
        .and_then(|c| c.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("canvas element")
}

#[test]
fn push_evicts_oldest_when_full() {
    let mut chart = StreamingBarChart::new(canvas(), 2);
    chart.push(bar(1, 10.0));
    chart.push(bar(2, 11.0));
    chart.push(bar(3, 12.0)); // evicts time 1
    assert_eq!(chart.len(), 2);
    assert_eq!(chart.bars()[0].time, 2);
    assert_eq!(chart.bars()[1].time, 3);
}

#[test]
fn update_last_revises_in_place() {
    let mut chart = StreamingBarChart::new(canvas(), 10);
    chart.set_bars(vec![bar(1, 10.0)]);
    chart.update_last(bar(1, 10.5));
    assert_eq!(chart.len(), 1);
    assert!((chart.bars()[0].close - 10.5).abs() < f64::EPSILON);
}
