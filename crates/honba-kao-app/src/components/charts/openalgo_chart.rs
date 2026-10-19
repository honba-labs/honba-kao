//! Rust ⇄ [openalgo-charts] bridge component.
//!
//! The engine is a dependency-free ESM bundle vendored under
//! `/vendor/openalgo-charts` and loaded through `honba-bridge.js`, which owns
//! the instances and maps the active CSS skin onto an openalgo `ChartTheme`.
//! This component mounts one chart host per use, pushes its series once, and
//! re-themes every chart when the app theme changes.
//!
//! [openalgo-charts]: https://github.com/marketcalls/openalgo-charts

use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::types::{Bar, SeriesKind};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = mount, catch)]
    pub(crate) fn js_mount(element: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setSeries, catch)]
    pub(crate) fn js_set_series(
        id: f64,
        kind: &str,
        style: &str,
        bars: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = updateLast, catch)]
    pub(crate) fn js_update_last(id: f64, bar: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addVolumeOverlay, catch)]
    pub(crate) fn js_add_volume_overlay(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addBuySell, catch)]
    pub(crate) fn js_add_buy_sell(id: f64, qty: f64, symbol: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setQuantity, catch)]
    pub(crate) fn js_set_quantity(id: f64, qty: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = destroy, catch)]
    pub(crate) fn js_destroy(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = indicatorCatalog, catch)]
    pub(crate) fn js_indicator_catalog() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = chartIndicators, catch)]
    pub(crate) fn js_chart_indicators(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addIndicator, catch)]
    pub(crate) fn js_add_indicator(
        id: f64,
        indicator_id: &str,
        settings_json: &str,
        instance_id: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = removeIndicator, catch)]
    pub(crate) fn js_remove_indicator(id: f64, instance_id: &str) -> Result<JsValue, JsValue>;
}

/// The full registry of built-in indicators, as decoded [`IndicatorDef`]s.
///
/// Shared with the workbench indicator picker (fetched once per session).
pub(crate) fn indicator_catalog() -> Result<Vec<IndicatorDef>, JsValue> {
    let json = js_indicator_catalog()?;
    serde_json::from_str::<Vec<IndicatorDef>>(json.as_string().unwrap_or_default().as_str())
        .map_err(|error| JsValue::from_str(&format!("indicator catalog: {error}")))
}

/// Live indicator instances currently running on one chart.
#[allow(dead_code)] // kept for console debugging / future diffing
pub(crate) fn chart_indicators(id: f64) -> Result<Vec<LiveIndicator>, JsValue> {
    let json = js_chart_indicators(id)?;
    serde_json::from_str::<Vec<LiveIndicator>>(json.as_string().unwrap_or_default().as_str())
        .map_err(|error| JsValue::from_str(&format!("live indicators: {error}")))
}

/// Add one built-in indicator to a chart, returning the live instance id.
pub(crate) fn add_indicator(
    id: f64,
    indicator_id: &str,
    settings: &serde_json::Value,
    instance_id: Option<&str>,
) -> Result<LiveIndicator, JsValue> {
    let settings_json =
        if settings.is_object() && !settings.as_object().is_some_and(|map| map.is_empty()) {
            settings.to_string()
        } else {
            String::new()
        };
    let json = js_add_indicator(id, indicator_id, &settings_json, instance_id.unwrap_or(""))?;
    json.as_string()
        .ok_or_else(|| JsValue::from_str("engine refused the indicator"))
        .and_then(|raw| {
            serde_json::from_str(&raw).map_err(|error| JsValue::from_str(&format!("{error}")))
        })
}

/// Remove one live indicator instance (idempotent).
pub(crate) fn remove_indicator(id: f64, instance_id: &str) {
    let _ = js_remove_indicator(id, instance_id);
}

/// Mount a chart host in the bridge, returning a numeric handle.
///
/// Shared with the workbench pane chart, which must remount charts as pane
/// symbol/timeframe/mode change (the engine has no clear-series call).
pub(crate) fn mount_chart(element: &JsValue) -> Result<f64, JsValue> {
    js_mount(element).and_then(|id| id.as_f64().ok_or_else(JsValue::undefined))
}

/// Push one series onto a mounted chart.
pub(crate) fn push_chart_series(
    id: f64,
    kind: &str,
    style: &str,
    bars: &str,
) -> Result<(), JsValue> {
    js_set_series(id, kind, style, bars).map(|_ok| ())
}

/// Update (or append) the in-flight bar of the live feed on a mounted chart.
pub(crate) fn update_chart_last(id: f64, bar: &Bar) -> bool {
    js_update_last(id, &serde_json::to_string(bar).unwrap_or_default())
        .map(|ok| ok.as_bool().unwrap_or(false))
        .unwrap_or(false)
}

/// Mount the volume histogram overlay in a chart's primary pane.
pub(crate) fn add_volume_overlay(id: f64) {
    let _ = js_add_volume_overlay(id);
}

/// Mount the corner Buy/Sell bubble; its clicks surface as
/// `honba-order-click` DOM events (see `order_click_stream`).
pub(crate) fn add_buy_sell_bubble(id: f64, quantity: i64, symbol: &str) {
    let _ = js_add_buy_sell(id, quantity as f64, symbol);
}

/// Keep the bubble's qty in step with the order ticket.
pub(crate) fn set_bubble_quantity(id: f64, quantity: i64) {
    let _ = js_set_quantity(id, quantity as f64);
}

/// Destroy a mounted chart (idempotent — the bridge answers `false`).
pub(crate) fn destroy_chart(id: f64) {
    let _ = js_destroy(id);
}

/// One series to draw, with the data the bridge should hand the engine.
#[derive(Clone, Debug)]
pub struct ChartSeries {
    pub kind: SeriesKind,
    /// A literal colour, or the name of a CSS token such as `"--primary-8"`.
    /// Tokens follow the active skin, so prefer them.
    pub color: &'static str,
    pub line_width: f64,
    pub bars: Vec<Bar>,
}

impl ChartSeries {
    pub fn candles(bars: Vec<Bar>) -> Self {
        Self {
            kind: SeriesKind::Candles,
            color: "",
            line_width: 0.0,
            bars,
        }
    }

    pub fn line(bars: Vec<Bar>, color: &'static str) -> Self {
        Self {
            kind: SeriesKind::Line,
            color,
            line_width: 2.0,
            bars,
        }
    }

    pub fn area(bars: Vec<Bar>) -> Self {
        Self {
            kind: SeriesKind::Area,
            color: "",
            line_width: 2.0,
            bars,
        }
    }
}

/// One tunable input of a built-in indicator (mirrors the engine's input spec).
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorInput {
    pub key: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub label: String,
    /// Literal default (`f64` / `bool` / `string` depending on `input_type`).
    #[serde(default)]
    pub default: serde_json::Value,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub group: Option<String>,
}

/// Options of a `select` input.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: serde_json::Value,
}

/// One row of the built-in indicator registry.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: String,
    /// `onchart` overlays share the price scale; `pane` get their own pane.
    #[serde(default)]
    pub placement: String,
    #[serde(default)]
    pub inputs: Vec<IndicatorInput>,
}

/// One live indicator instance running on a chart.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveIndicator {
    /// The instance id the engine minted (also what `removeIndicator` wants).
    pub instance_id: String,
    /// The registry id (`sma`, `rsi`, …).
    #[serde(default)]
    pub indicator_id: String,
    /// Human name for the pane header chip.
    #[serde(default)]
    pub name: String,
}

/// An openalgo-charts canvas bound to a `<div>`.
///
/// `class` carries the sizing (the host needs a height); `label` is the
/// accessible name for the canvas.
#[component]
pub fn OpenAlgoChart(
    series: Vec<ChartSeries>,
    #[prop(optional, into)] class: String,
    #[prop(default = "Price chart")] label: &'static str,
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();
    let handle = RwSignal::new(None::<f64>);
    let attempts = RwSignal::new(0u32);

    // Serialised once: the demo data is static per mount, and the bridge wants
    // strings either way.
    let payload = StoredValue::new(serialize_series(&series));

    Effect::new(move |_| {
        let _ = attempts.get();

        let Some(element) = host.get() else {
            return;
        };
        if handle.get_untracked().is_some() {
            return;
        }

        match js_mount(&JsValue::from(element)) {
            Ok(id) => {
                let id = id.as_f64().unwrap_or_default();
                for (kind, style, bars) in payload.get_value() {
                    let _ = js_set_series(id, &kind, &style, &bars);
                }
                handle.set(Some(id));
            }
            // The ESM module may still be fetching; retry briefly, then give up
            // quietly rather than panic the app.
            Err(_) if attempts.get_untracked() < 120 => {
                set_timeout(
                    move || attempts.update(|count| *count += 1),
                    Duration::from_millis(25),
                );
            }
            Err(error) => {
                leptos::logging::error!("openalgo-charts bridge unavailable: {error:?}");
            }
        }
    });

    // Any skin change restyles every live chart: `ThemeContext::apply` asks the
    // bridge to re-read the tokens, so this component does not subscribe.
    on_cleanup(move || {
        if let Some(id) = handle.get_untracked() {
            let _ = js_destroy(id);
        }
    });

    view! { <div class=class node_ref=host role="img" aria-label=label /> }
}

fn serialize_series(series: &[ChartSeries]) -> Vec<(String, String, String)> {
    series.iter().map(encode_series).collect()
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

/// Encode just the OHLCV bars (the workbench builds kind/style itself so all
/// thirteen engine chart kinds are reachable).
pub(crate) fn encode_bars(bars: &[Bar]) -> String {
    serde_json::to_string(bars).unwrap_or_else(|_| "[]".to_string())
}
