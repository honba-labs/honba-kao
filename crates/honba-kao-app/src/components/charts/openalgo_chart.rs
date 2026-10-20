//! Rust ⇄ [openalgo-charts widget] bridge component.
//!
//! The widget tier is a dependency-free ESM bundle vendored under
//! `/vendor/openalgo-charts` and loaded through `honba-bridge.js`, which owns
//! the widget instances and maps the active CSS skin onto an openalgo `ChartTheme`.
//! This component mounts one widget per use, and the widget provides the full
//! OpenAlgo terminal: topbar, drawing rail, dialogs, indicator picker (502),
//! objects panel, settings, snapshot, and layout persistence.
//!
//! [openalgo-charts]: https://github.com/marketcalls/openalgo-charts

use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::types::{Bar, SeriesKind};

#[wasm_bindgen]
extern "C" {
    // `catch`: until the bridge module has loaded, `window.honbaCharts` is
    // undefined and the call throws; that must surface as an `Err` (so the
    // mount effects retry) rather than a JS exception unwinding through wasm.
    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = mount, catch)]
    pub(crate) fn js_mount_with_options(
        element: &JsValue,
        symbol: &str,
        timeframe: &str,
        chart_mode: &str,
        quantity: f64,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = mountStatic, catch)]
    fn js_mount_static(element: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setSeries, catch)]
    fn js_set_series(id: f64, kind: &str, style: &str, bars: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addVolumeOverlay, catch)]
    pub(crate) fn js_add_volume_overlay(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = addBuySell, catch)]
    pub(crate) fn js_add_buy_sell(id: f64, qty: f64, symbol: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setQuantity, catch)]
    pub(crate) fn js_set_quantity(id: f64, qty: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setSymbolContext, catch)]
    pub(crate) fn js_set_symbol_context(id: f64, symbol: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setDrawingTool, catch)]
    pub(crate) fn js_set_drawing_tool(id: f64, tool: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = drawingTool, catch)]
    pub(crate) fn js_drawing_tool(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = undoDrawing, catch)]
    pub(crate) fn js_undo_drawing(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = redoDrawing, catch)]
    pub(crate) fn js_redo_drawing(id: f64) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = clearDrawings, catch)]
    pub(crate) fn js_clear_drawings(id: f64) -> Result<JsValue, JsValue>;

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

    // Widget-tier APIs (indicators / settings / objects / capture / chart
    // type now come from the widget's native top bar, so no bindings here).
    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = registerCustomIndicatorPicker, catch)]
    pub(crate) fn js_register_custom_indicator_picker(mount_fn: &JsValue) -> Result<JsValue, JsValue>;
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

/// Publish the pane's instrument so drawings persist per symbol
/// (`exchange:symbol` key in localStorage, re-loaded on every remount).
pub(crate) fn set_symbol_context(id: f64, symbol: &str) {
    let _ = js_set_symbol_context(id, symbol);
}

/// Arm a drawing tool (`""` = cursor), returning whether the engine took it.
pub(crate) fn set_drawing_tool(id: f64, tool: &str) -> bool {
    js_set_drawing_tool(id, tool)
        .map(|ok| ok.as_bool().unwrap_or(false))
        .unwrap_or(false)
}

/// The chart's active drawing tool id (`"cursor"` when none is armed).
#[allow(dead_code)]
pub(crate) fn drawing_tool(id: f64) -> String {
    js_drawing_tool(id)
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "cursor".to_string())
}

pub(crate) fn undo_drawing(id: f64) {
    let _ = js_undo_drawing(id);
}

pub(crate) fn redo_drawing(id: f64) {
    let _ = js_redo_drawing(id);
}

pub(crate) fn clear_drawings(id: f64) {
    let _ = js_clear_drawings(id);
}

/// Destroy a mounted chart (idempotent — the bridge answers `false`).
pub(crate) fn destroy_chart(id: f64) {
    let _ = js_destroy(id);
}

/// Mount a chart host with widget options (symbol, timeframe, chart_mode, quantity).
///
/// The widget tier creates the full terminal: chart + rail + topbar + statusline.
/// `mount()` on the JS side is async (it awaits `widget.ready`), so this awaits
/// the returned Promise rather than treating the call as synchronous.
pub(crate) async fn mount_chart_with_options(
    element: &JsValue,
    symbol: &str,
    timeframe: &str,
    chart_mode: &str,
    quantity: i64,
) -> Result<f64, JsValue> {
    let promise = js_mount_with_options(element, symbol, timeframe, chart_mode, quantity as f64)?;
    let resolved = JsFuture::from(promise).await?;
    resolved.as_f64().ok_or_else(JsValue::undefined)
}

/// Register a custom indicator picker dialog (replaces the built-in one).
pub(crate) fn register_custom_indicator_picker(mount_fn: &JsValue) {
    let _ = js_register_custom_indicator_picker(mount_fn);
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

/// The OpenAlgoChart widget component — uses the widget tier for full terminal experience.
#[component]
pub fn OpenAlgoChart(
    #[prop(optional)] symbol: Option<String>,
    #[prop(optional)] timeframe: Option<String>,
    #[prop(optional)] chart_mode: Option<String>,
    #[prop(optional)] quantity: Option<i64>,
    #[prop(optional)] series: Option<Vec<ChartSeries>>,
    #[prop(optional, into)] class: String,
    #[prop(default = "Price chart")] label: &'static str,
) -> impl IntoView {
    let symbol = symbol.unwrap_or_else(|| "RELIANCE".to_string());
    let timeframe = timeframe.unwrap_or_else(|| "1d".to_string());
    let chart_mode = chart_mode.unwrap_or_else(|| "candlestick".to_string());
    let quantity = quantity.unwrap_or(25);
    let series = series.unwrap_or_default();
    // Callers that hand over their own series (equity curve, comparison)
    // get a plain engine chart with exactly that data, not the feed-backed
    // terminal widget.
    let is_static = !series.is_empty();

    let host = NodeRef::<leptos::html::Div>::new();
    let handle = RwSignal::new(None::<f64>);
    let attempts = RwSignal::new(0u32);
    let alive = RwSignal::new(true);

    let payload = StoredValue::new(serialize_series(&series));

    Effect::new(move |_| {
        let _ = attempts.get();

        let Some(element) = host.get() else {
            return;
        };
        if handle.get_untracked().is_some() {
            return;
        }

        if is_static {
            match js_mount_static(&JsValue::from(element)) {
                Ok(id) => {
                    let id = id.as_f64().unwrap_or_default();
                    for (kind, style, bars) in payload.get_value() {
                        let _ = js_set_series(id, &kind, &style, &bars);
                    }
                    handle.set(Some(id));
                }
                // The ESM module may still be fetching; retry briefly.
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
            return;
        }

        let symbol = symbol.clone();
        let timeframe = timeframe.clone();
        let chart_mode = chart_mode.clone();
        let element = JsValue::from(element);

        // The mount resolves asynchronously, possibly after this component
        // (and with it these signals) was disposed: `try_get_untracked`, as a
        // plain `get_untracked` on a disposed signal panics.
        leptos::task::spawn_local(async move {
            match mount_chart_with_options(&element, &symbol, &timeframe, &chart_mode, quantity).await {
                Ok(id) => {
                    if alive.try_get_untracked().unwrap_or(false) {
                        handle.set(Some(id));
                    } else {
                        destroy_chart(id);
                    }
                }
                Err(_) if attempts.try_get_untracked().is_some_and(|count| count < 120) => {
                    set_timeout(
                        move || attempts.update(|count| *count += 1),
                        Duration::from_millis(25),
                    );
                }
                Err(error) => {
                    leptos::logging::error!("openalgo-charts widget unavailable: {error:?}");
                }
            }
        });
    });

    on_cleanup(move || {
        alive.set(false);
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
