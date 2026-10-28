//! Reactive Leptos wrapper for the OpenAlgo Charts canvas engine.
//!
//! `<OpenalgoChart />` mounts a `window.honbaCharts.mountStatic` chart on a
//! `<canvas>` and pushes `CandleData` through `setSeries` on load and
//! `updateLast` for live ticks — an ergonomic reactive signal-driven chart.

use crate::canvas::bridge;
use crate::canvas::engine::{SeriesKind, SeriesStyle};
use crate::theme::ChartTheme;
use crate::types::CandleData;
use leptos::prelude::*;

/// Ergonomic reactive chart component.
///
/// - `data`: reactive candle stream; the full history is pushed on mount and
///   each subsequent change streams the latest bar via `updateLast`.
/// - `theme`: optional chart palette; falls back to the engine default.
/// - `height`: CSS height, defaults to `"100%"`.
/// - `kind`: `"candlestick"` (default), `"bar"`, `"line"`, or `"area"`.
#[component]
pub fn OpenalgoChart(
    #[prop(into)] data: Signal<Vec<CandleData>>,
    #[prop(into, optional)] theme: Option<ChartTheme>,
    #[prop(into, optional)] height: Option<String>,
    #[prop(into, optional)] kind: Option<String>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let chart_id = RwSignal::new(None::<f64>);
    // `initialized` means "the initial full history has been committed to the
    // engine"; afterwards only the last bar is streamed on each change.
    let initialized = RwSignal::new(false);

    let height = height.unwrap_or_else(|| "100%".to_string());
    let kind = kind.unwrap_or_else(|| "candlestick".to_string());
    let series_kind = match kind.as_str() {
        "bar" => SeriesKind::Bar,
        "line" => SeriesKind::Line,
        "area" => SeriesKind::Area,
        _ => SeriesKind::Candlestick,
    };
    let style = SeriesStyle {
        up_color: theme.as_ref().map(|t| t.up_color.clone()),
        down_color: theme.as_ref().map(|t| t.down_color.clone()),
        color: theme.as_ref().map(|t| t.up_color.clone()),
        line_width: Some(2.0),
    };
    let style_json = serde_json::to_string(&style).unwrap_or_default();

    Effect::new(move |_| {
        let canvas = canvas_ref.get();
        let data = data.get();
        let Some(canvas) = canvas else {
            return;
        };

        // Obtain (or create) the engine chart id for this canvas.
        let id = if chart_id.get().is_none() {
            match bridge::mount_static(&canvas) {
                Ok(id) => {
                    chart_id.set(Some(id));
                    id
                }
                Err(_) => return, // engine not available outside a browser
            }
        } else {
            chart_id.get().expect("chart id set once mounted")
        };

        if data.is_empty() {
            return;
        }

        let bars_json = serde_json::to_string(&data).unwrap_or_default();
        if !initialized.get() {
            // First mount: push the full history.
            let _ = bridge::set_series(id, series_kind.as_str(), &style_json, &bars_json);
            initialized.set(true);
        } else {
            // Live stream: revise/append only the last bar.
            if let Some(last) = data.last() {
                let bar_json = serde_json::to_string(last).unwrap_or_default();
                let _ = bridge::update_last(id, &bar_json);
            }
        }
    });

    view! {
        <canvas
            node_ref=canvas_ref
            class="openalgo-chart w-full h-full bg-surface-base block"
            style=format!("height: {}; width: 100%; display: block;", height)
            aria-label="financial chart"
        ></canvas>
    }
}
