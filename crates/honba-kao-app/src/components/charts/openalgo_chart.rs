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
    fn js_mount(element: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = setSeries, catch)]
    fn js_set_series(id: f64, kind: &str, style: &str, bars: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "honbaCharts"], js_name = destroy, catch)]
    fn js_destroy(id: f64) -> Result<JsValue, JsValue>;
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
    series
        .iter()
        .map(|series| {
            let style = serde_json::json!({
                "color": series.color,
                "lineWidth": series.line_width,
            });
            let bars = serde_json::to_string(&series.bars).unwrap_or_else(|_| "[]".to_string());

            (series.kind.as_str().to_string(), style.to_string(), bars)
        })
        .collect()
}
