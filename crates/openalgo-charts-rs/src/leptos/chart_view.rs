//! Reactive Leptos wrapper for the OpenAlgo Charts canvas engine.
//!
//! `<OpenalgoChart />` mounts a `window.honbaCharts.mountStatic` chart on a
//! `<canvas>` and pushes `CandleData` through `setSeries` on load. Each
//! subsequent `data` change is inspected: a true single-bar append or a
//! revision of the currently-forming last bar streams through the cheap
//! `updateLast` path, while a wholesale replacement (symbol switch, history
//! refetch, or a `kind`/`theme` change) re-pushes the full history through
//! `setSeries` so stale history never lingers behind a fresh last bar.
//! The mounted chart is destroyed on unmount.

use crate::canvas::bridge;
use crate::canvas::engine::{SeriesKind, SeriesStyle};
use crate::theme::ChartTheme;
use crate::types::CandleData;
use leptos::prelude::*;

/// Ergonomic reactive chart component.
///
/// - `data`: reactive candle stream; the full history is pushed on mount and
///   on any wholesale replacement, and each true single-bar append/revision
///   streams via `updateLast`.
/// - `theme`: optional, reactive chart palette; falls back to the engine
///   default. Changing it re-applies the engine's active skin and re-pushes
///   the series with the new colors.
/// - `height`: CSS height, defaults to `"100%"`.
/// - `kind`: optional, reactive `"candlestick"` (default), `"bar"`, `"line"`,
///   or `"area"`. Changing it re-renders the series as the new kind.
#[component]
pub fn OpenalgoChart(
    #[prop(into)] data: Signal<Vec<CandleData>>,
    #[prop(into, optional)] theme: Signal<Option<ChartTheme>>,
    #[prop(into, optional)] height: Option<String>,
    #[prop(into, optional)] kind: Signal<Option<String>>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let chart_id = RwSignal::new(None::<f64>);
    // `initialized` means "the initial full history has been committed to the
    // engine"; afterwards each change is classified as a cheap incremental
    // update or a full replacement (see the effect below).
    let initialized = RwSignal::new(false);

    // Non-reactive bookkeeping used to classify each `data`/`kind`/`theme`
    // change without re-running the effect merely because we wrote to them.
    let rendered_data = StoredValue::new_local(Vec::<CandleData>::new());
    let last_kind = StoredValue::new_local(None::<SeriesKind>);
    let last_theme = StoredValue::new_local(None::<ChartTheme>);

    let height = height.unwrap_or_else(|| "100%".to_string());

    Effect::new(move |_| {
        let canvas = canvas_ref.get();
        let data = data.get();
        let kind_value = kind.get();
        let theme_value = theme.get();
        let Some(canvas) = canvas else {
            return;
        };

        // `chart_id` is written at most once by this effect (on first
        // mount) and must never become a reactive dependency of it, so read
        // it untracked.
        let id = if chart_id.get_untracked().is_none() {
            match bridge::mount_static(&canvas) {
                Ok(id) => {
                    chart_id.set(Some(id));
                    id
                }
                Err(_) => return, // engine not available outside a browser
            }
        } else {
            chart_id.get_untracked().expect("chart id set once mounted")
        };

        if data.is_empty() {
            return;
        }

        let series_kind = match kind_value.as_deref() {
            Some("bar") => SeriesKind::Bar,
            Some("line") => SeriesKind::Line,
            Some("area") => SeriesKind::Area,
            _ => SeriesKind::Candlestick,
        };
        let style = SeriesStyle {
            up_color: theme_value.as_ref().map(|t| t.up_color.clone()),
            down_color: theme_value.as_ref().map(|t| t.down_color.clone()),
            color: theme_value.as_ref().map(|t| t.up_color.clone()),
            line_width: Some(2.0),
        };
        let style_json = serde_json::to_string(&style).unwrap_or_default();

        let is_first_render = !initialized.get_untracked();
        let kind_changed = !is_first_render && last_kind.get_value() != Some(series_kind);
        let theme_changed = !is_first_render && last_theme.get_value() != theme_value;

        if theme_changed {
            // The engine's "active skin" (grid/background chrome) is a
            // separate, global concept from the per-series `style` above;
            // re-apply it whenever the chart's theme prop changes.
            let _ = bridge::refresh_theme_all();
        }

        // Classify this change: a true single-bar append, or a revision of
        // the currently-forming last bar, keeps everything before the last
        // element identical to what the engine already has, so it can
        // stream through the cheap `updateLast` path. Anything else — a
        // shrunk/replaced history, a kind/theme change, or more than one new
        // bar at once — means the engine's history is stale and must be
        // re-pushed in full.
        let is_incremental_update = !is_first_render
            && !kind_changed
            && !theme_changed
            && rendered_data.with_value(|prev| {
                let prev_len = prev.len();
                let new_len = data.len();
                if new_len == prev_len && prev_len > 0 {
                    data[..new_len - 1] == prev[..prev_len - 1]
                } else if new_len == prev_len + 1 {
                    data[..prev_len] == prev[..]
                } else {
                    false
                }
            });

        if is_incremental_update {
            // Live stream: revise/append only the last bar.
            if let Some(last) = data.last() {
                let bar_json = serde_json::to_string(last).unwrap_or_default();
                let _ = bridge::update_last(id, &bar_json);
            }
        } else {
            // First mount, kind/theme change, or wholesale replacement:
            // push the full history.
            let bars_json = serde_json::to_string(&data).unwrap_or_default();
            let _ = bridge::set_series(id, series_kind.as_str(), &style_json, &bars_json);
        }

        initialized.set(true);
        last_kind.set_value(Some(series_kind));
        last_theme.set_value(theme_value);
        rendered_data.set_value(data);
    });

    on_cleanup(move || {
        if let Some(id) = chart_id.get_untracked() {
            let _ = bridge::destroy(id);
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
