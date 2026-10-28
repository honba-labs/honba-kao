//! Focused ChartContainer component for pure OpenAlgo / Lightweight charts.

use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::chart::ChartHandle;
use crate::events::{CrosshairMoveEvent, Subscription};
use crate::types::{Bar, ChartOptions, SeriesKind, SeriesOptions};

/// A lean, dedicated container component that manages a `createChart` instance.
///
/// Follows the idiomatic `rs-trading-charts` pattern with full lifecycle management,
/// reactive signal propagation, and zero memory leaks.
#[component]
pub fn ChartContainer(
    /// Reactive bar data signal.
    #[prop(into)]
    data: Signal<Vec<Bar>>,
    /// Series rendering kind (default: Candlestick).
    #[prop(optional)]
    series_kind: Option<SeriesKind>,
    /// Global chart options.
    #[prop(optional)]
    options: Option<ChartOptions>,
    /// Series style options.
    #[prop(optional)]
    series_options: Option<SeriesOptions>,
    /// CSS class name for the wrapper <div>.
    #[prop(optional, into)]
    class: Option<String>,
    /// CSS inline style for the wrapper <div>.
    #[prop(optional, into)]
    style: Option<String>,
    /// Optional external signal to receive the ChartHandle once mounted.
    #[prop(optional)]
    chart_ref: Option<RwSignal<Option<ChartHandle>>>,
    /// Optional callback fired on crosshair movement.
    #[prop(optional)]
    on_crosshair: Option<Callback<CrosshairMoveEvent>>,
    /// Optional callback fired when a chart element is clicked.
    #[prop(optional)]
    on_click: Option<Callback<String>>,
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();
    let chart_store = StoredValue::<Option<ChartHandle>>::new(None);
    let series_store = StoredValue::<Option<crate::chart::SeriesHandle>>::new(None);
    let crosshair_sub = StoredValue::<Option<Subscription>>::new(None);
    let click_sub = StoredValue::<Option<Subscription>>::new(None);
    let attempts = RwSignal::new(0u32);

    let kind = series_kind.unwrap_or(SeriesKind::Candlestick);
    let class_str = class.unwrap_or_else(|| "openalgo-chart-container".to_string());
    let style_str = style.unwrap_or_else(|| "width: 100%; height: 100%;".to_string());

    // ------------------------------------------------------------------------
    // Mount Lifecycle: createChart + addSeries + initial setData
    // ------------------------------------------------------------------------
    Effect::new(move |_| {
        let _ = attempts.get();

        if chart_store.get_value().is_some() {
            return;
        }

        let Some(element) = host.get() else {
            return;
        };

        let html_el: web_sys::HtmlElement = element.unchecked_into();

        match ChartHandle::create(&html_el, options.as_ref()) {
            Ok(handle) => {
                match handle.add_series(kind, series_options.as_ref()) {
                    Ok(series_handle) => {
                        let initial_bars = data.get();
                        let _ = series_handle.set_data(&initial_bars);
                        series_store.set_value(Some(series_handle));
                    }
                    Err(e) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Failed to add primary series: {e:?}");
                    }
                }

                if let Some(cb) = on_crosshair {
                    let sub = handle.subscribe_crosshair_move(move |ev| {
                        cb.run(ev);
                    });
                    crosshair_sub.set_value(sub);
                }

                if let Some(cb) = on_click {
                    let sub = handle.subscribe_click(move |id| {
                        cb.run(id);
                    });
                    click_sub.set_value(sub);
                }

                handle.fit_content();

                if let Some(cr) = chart_ref {
                    cr.set(Some(handle.clone()));
                }
                chart_store.set_value(Some(handle));
            }
            Err(_) if attempts.get_untracked() < 120 => {
                set_timeout(
                    move || attempts.update(|count| *count += 1),
                    Duration::from_millis(25),
                );
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Failed to create chart: {e:?}");
            }
        }
    });

    // ------------------------------------------------------------------------
    // Reactivity: update chart data when the `data` signal changes
    // ------------------------------------------------------------------------
    Effect::new(move |_| {
        let new_bars = data.get();
        if let Some(series) = series_store.get_value() {
            let _ = series.set_data(&new_bars);
        }
    });

    // ------------------------------------------------------------------------
    // Lifecycle: Cleanup on unmount
    // ------------------------------------------------------------------------
    on_cleanup(move || {
        crosshair_sub.update_value(|s| {
            if let Some(sub) = s.take() {
                sub.unsubscribe();
            }
        });
        click_sub.update_value(|s| {
            if let Some(sub) = s.take() {
                sub.unsubscribe();
            }
        });
        if let Some(handle) = chart_store.get_value() {
            handle.destroy();
        }
    });

    view! {
        <div
            class=class_str
            style=style_str
            node_ref=host
        />
    }
}
