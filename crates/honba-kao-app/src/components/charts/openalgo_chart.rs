//! Rust ⇄ [openalgo-charts] bridge component.
//!
//! Re-exports and integrates the factored-out `openalgo-charts` crate for honba-kao.

use wasm_bindgen::prelude::*;

// Re-export core types and components from openalgo-charts
pub use openalgo_charts::chart::{ChartError, ChartHandle, SeriesHandle};
pub use openalgo_charts::drawing::DrawingTool;
pub use openalgo_charts::indicators::{IndicatorDef, IndicatorInput, LiveIndicator, SelectOption};
pub use openalgo_charts::types::*;

pub use openalgo_charts::OpenAlgoChart;

/// Mount a chart host with widget options (symbol, timeframe, chart_mode, quantity).
pub(crate) async fn mount_chart_with_options(
    element: &JsValue,
    symbol: &str,
    timeframe: &str,
    chart_mode: &str,
    quantity: i64,
) -> Result<f64, JsValue> {
    let promise = openalgo_charts::sys::bridge_mount_with_options(
        element,
        symbol,
        timeframe,
        chart_mode,
        quantity as f64,
    )?;
    let resolved = wasm_bindgen_futures::JsFuture::from(promise).await?;
    resolved.as_f64().ok_or_else(JsValue::undefined)
}

/// The full registry of built-in indicators, as decoded [`IndicatorDef`]s.
pub(crate) fn indicator_catalog() -> Result<Vec<IndicatorDef>, JsValue> {
    openalgo_charts::indicators::indicator_catalog().map_err(|e| JsValue::from_str(&e))
}

/// Live indicator instances currently running on one chart.
#[allow(dead_code)]
pub(crate) fn chart_indicators(id: f64) -> Result<Vec<LiveIndicator>, JsValue> {
    openalgo_charts::indicators::chart_indicators(id).map_err(|e| JsValue::from_str(&e))
}

/// Add one built-in indicator to a chart, returning the live instance id.
pub(crate) fn add_indicator(
    id: f64,
    indicator_id: &str,
    settings: &serde_json::Value,
    instance_id: Option<&str>,
) -> Result<LiveIndicator, JsValue> {
    openalgo_charts::indicators::add_indicator(id, indicator_id, settings, instance_id)
        .map_err(|e| JsValue::from_str(&e))
}

/// Remove one live indicator instance (idempotent).
pub(crate) fn remove_indicator(id: f64, instance_id: &str) {
    openalgo_charts::indicators::remove_indicator(id, instance_id);
}

/// Mount the volume histogram overlay in a chart's primary pane.
pub(crate) fn add_volume_overlay(id: f64) {
    let _ = openalgo_charts::sys::bridge_add_volume_overlay(id);
}

/// Mount the corner Buy/Sell bubble; its clicks surface as `honba-order-click` DOM events.
pub(crate) fn add_buy_sell_bubble(id: f64, quantity: i64, symbol: &str) {
    let _ = openalgo_charts::sys::bridge_add_buy_sell(id, quantity as f64, symbol);
}

/// Keep the bubble's qty in step with the order ticket.
pub(crate) fn set_bubble_quantity(id: f64, quantity: i64) {
    let _ = openalgo_charts::sys::bridge_set_quantity(id, quantity as f64);
}

/// Publish the pane's instrument so drawings persist per symbol.
pub(crate) fn set_symbol_context(id: f64, symbol: &str) {
    openalgo_charts::drawing::set_symbol_context(id, symbol);
}

pub use openalgo_charts::drawing::{
    clear_drawings, drawing_tool, redo_drawing, set_drawing_tool, undo_drawing,
};
pub use openalgo_charts::indicators::register_custom_indicator_picker;

/// Destroy a mounted chart (idempotent).
pub(crate) fn destroy_chart(id: f64) {
    let _ = openalgo_charts::sys::bridge_destroy(id);
}

/// Encode one series into the `(kind, style, bars)` JSON triple the bridge's
/// `setSeries` accepts. Shared with the workbench pane chart.
pub(crate) fn encode_series(series: &ChartSeries) -> (String, String, String) {
    let style = serde_json::json!({
        "color": series.color,
        "lineWidth": series.line_width,
    });
    (
        series.kind.as_str().to_string(),
        style.to_string(),
        encode_bars(&series.bars),
    )
}

/// Encode just the OHLCV bars.
pub(crate) fn encode_bars(bars: &[Bar]) -> String {
    serde_json::to_string(bars).unwrap_or_else(|_| "[]".to_string())
}
