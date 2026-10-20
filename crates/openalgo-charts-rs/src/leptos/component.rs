//! Leptos components for OpenAlgo Charts.

use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::chart::ChartHandle;
use crate::events::{CrosshairMoveEvent, OrderClickEvent, Subscription};
use crate::sys;
use crate::types::{Bar, ChartOptions, ChartSeries, SeriesKind, SeriesOptions};

/// Primary OpenAlgo chart component.
///
/// Supports both:
/// 1. Pure engine charts with host data (`series` or `data` signal) via `createChart`.
/// 2. Widget-tier full terminal experience with native topbar, drawing rail, and live feeds.
#[component]
pub fn OpenAlgoChart(
    #[prop(optional)] symbol: Option<String>,
    #[prop(optional)] timeframe: Option<String>,
    #[prop(optional)] chart_mode: Option<String>,
    #[prop(optional)] quantity: Option<i64>,
    #[prop(optional)] series: Option<Vec<ChartSeries>>,
    #[prop(optional)] data: Option<Signal<Vec<Bar>>>,
    #[prop(optional)] options: Option<ChartOptions>,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] style: Option<String>,
    #[prop(default = "Price chart")] label: &'static str,
    #[prop(optional)] chart_handle: Option<RwSignal<Option<ChartHandle>>>,
    #[prop(optional)] on_crosshair: Option<Callback<CrosshairMoveEvent>>,
    #[prop(optional)] on_order_click: Option<Callback<OrderClickEvent>>,
) -> impl IntoView {
    let symbol = symbol.unwrap_or_else(|| "RELIANCE".to_string());
    let timeframe = timeframe.unwrap_or_else(|| "1d".to_string());
    let chart_mode = chart_mode.unwrap_or_else(|| "candlestick".to_string());
    let quantity = quantity.unwrap_or(25);
    let initial_series = series.unwrap_or_default();

    let is_static = !initial_series.is_empty() || data.is_some();

    let host = NodeRef::<leptos::html::Div>::new();
    let chart_instance = StoredValue::<Option<ChartHandle>>::new(None);
    let attempts = RwSignal::new(0u32);
    let alive = RwSignal::new(true);

    // Track active series for reactive data updates
    let primary_series = StoredValue::<Option<crate::chart::SeriesHandle>>::new(None);
    let crosshair_sub = StoredValue::<Option<Subscription>>::new(None);

    let class_str = class.unwrap_or_default();
    let style_str = style.unwrap_or_default();

    // ------------------------------------------------------------------------
    // Chart Mount Lifecycle
    // ------------------------------------------------------------------------
    Effect::new(move |_| {
        let _ = attempts.get();

        let Some(element) = host.get() else {
            return;
        };

        if chart_instance.get_value().is_some() {
            return;
        }

        let html_el: web_sys::HtmlElement = element.clone().unchecked_into();

        if is_static {
            // Static mode: Try direct createChart, fallback to bridge mountStatic
            let created = ChartHandle::create(&html_el, options.as_ref()).or_else(|_| {
                sys::bridge_mount_static(&JsValue::from(html_el.clone()))
                    .map(|id| ChartHandle::from_bridge_id(id.as_f64().unwrap_or_default()))
                    .map_err(|e| crate::chart::ChartError::Js(format!("{e:?}")))
            });

            match created {
                Ok(handle) => {
                    // Populate initial series if provided
                    for s in &initial_series {
                        let opts = SeriesOptions {
                            color: if s.color.is_empty() {
                                None
                            } else {
                                Some(s.color.clone())
                            },
                            line_width: Some(s.line_width),
                            ..Default::default()
                        };
                        if let Ok(sh) = handle.add_series(s.kind, Some(&opts)) {
                            let _ = sh.set_data(&s.bars);
                            if primary_series.get_value().is_none() {
                                primary_series.set_value(Some(sh));
                            }
                        }
                    }

                    // If a reactive data signal was provided and no initial series, add default primary series
                    if initial_series.is_empty() && data.is_some() {
                        let kind = SeriesKind::from_str_name(&chart_mode);
                        if let Ok(sh) = handle.add_series(kind, None) {
                            if let Some(data_sig) = data {
                                let _ = sh.set_data(&data_sig.get());
                            }
                            primary_series.set_value(Some(sh));
                        }
                    }

                    // Register crosshair listener if requested
                    if let Some(cb) = on_crosshair {
                        let sub = handle.subscribe_crosshair_move(move |ev| {
                            cb.run(ev);
                        });
                        crosshair_sub.set_value(sub);
                    }

                    handle.fit_content();

                    if let Some(ch) = chart_handle {
                        ch.set(Some(handle.clone()));
                    }
                    chart_instance.set_value(Some(handle));
                }
                Err(_) if attempts.get_untracked() < 120 => {
                    set_timeout(
                        move || attempts.update(|count| *count += 1),
                        Duration::from_millis(25),
                    );
                }
                Err(error) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("openalgo-charts engine initialization failed: {error:?}");
                }
            }
            return;
        }

        // Terminal Widget mode: mounts with symbol, timeframe, indicators, and tools
        let symbol_clone = symbol.clone();
        let timeframe_clone = timeframe.clone();
        let chart_mode_clone = chart_mode.clone();
        let element_js = JsValue::from(html_el);

        leptos::task::spawn_local(async move {
            match sys::bridge_mount_with_options(
                &element_js,
                &symbol_clone,
                &timeframe_clone,
                &chart_mode_clone,
                quantity as f64,
            ) {
                Ok(promise) => {
                    match JsFuture::from(promise).await {
                        Ok(resolved) => {
                            let id = resolved.as_f64().unwrap_or_default();
                            if alive.try_get_untracked().unwrap_or(false) {
                                let handle = ChartHandle::from_bridge_id(id);
                                if let Some(ch) = chart_handle {
                                    ch.set(Some(handle.clone()));
                                }
                                chart_instance.set_value(Some(handle));
                            } else {
                                let _ = sys::bridge_destroy(id);
                            }
                        }
                        Err(_) if attempts.try_get_untracked().is_some_and(|c| c < 120) => {
                            set_timeout(
                                move || attempts.update(|c| *c += 1),
                                Duration::from_millis(25),
                            );
                        }
                        Err(e) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("openalgo widget failed: {e:?}");
                        }
                    }
                }
                Err(_) if attempts.try_get_untracked().is_some_and(|c| c < 120) => {
                    set_timeout(
                        move || attempts.update(|c| *c += 1),
                        Duration::from_millis(25),
                    );
                }
                Err(e) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("bridge_mount_with_options error: {e:?}");
                }
            }
        });
    });

    // ------------------------------------------------------------------------
    // Reactivity: update data bars when signal changes
    // ------------------------------------------------------------------------
    if let Some(data_sig) = data {
        Effect::new(move |_| {
            let bars = data_sig.get();
            if let Some(sh) = primary_series.get_value() {
                let _ = sh.set_data(&bars);
            }
        });
    }

    // ------------------------------------------------------------------------
    // Reactivity: Buy/Sell Order Clicks
    // ------------------------------------------------------------------------
    if let Some(cb) = on_order_click {
        let order_listener = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            let Ok(custom_ev) = event.dyn_into::<web_sys::CustomEvent>() else {
                return;
            };
            let detail = custom_ev.detail();
            let side = js_sys::Reflect::get(&detail, &JsValue::from_str("side"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let sym = js_sys::Reflect::get(&detail, &JsValue::from_str("symbol"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let qty = js_sys::Reflect::get(&detail, &JsValue::from_str("qty"))
                .ok()
                .and_then(|v| v.as_f64())
                .map(|q| q as i64)
                .unwrap_or(0);

            cb.run(OrderClickEvent {
                side,
                symbol: sym,
                quantity: qty,
            });
        });

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "honba-order-click",
                order_listener.as_ref().unchecked_ref(),
            );
        }
    }

    // ------------------------------------------------------------------------
    // Lifecycle Cleanup
    // ------------------------------------------------------------------------
    on_cleanup(move || {
        alive.set(false);
        // Clean up crosshair subscription
        crosshair_sub.update_value(|s| {
            if let Some(sub) = s.take() {
                sub.unsubscribe();
            }
        });
        // Destroy chart instance to prevent memory leaks
        if let Some(handle) = chart_instance.get_value() {
            handle.destroy();
        }
    });

    view! {
        <div
            class=class_str
            style=style_str
            node_ref=host
            role="img"
            aria-label=label
        />
    }
}
