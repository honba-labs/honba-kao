//! High-level, type-safe Chart and Series abstractions.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::events::{CrosshairMoveEvent, Subscription};
use crate::indicators::LiveIndicator;
use crate::sys;
use crate::types::{Bar, ChartOptions, LineData, SeriesKind, SeriesOptions};

#[derive(thiserror::Error, Debug)]
pub enum ChartError {
    #[error("JavaScript error: {0}")]
    Js(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Chart instance has already been destroyed")]
    Destroyed,
}

impl From<JsValue> for ChartError {
    fn from(val: JsValue) -> Self {
        Self::Js(format!("{val:?}"))
    }
}

/// A high-level handle to a chart instance.
///
/// Can manage an imperative `sys::Chart` engine instance or a widget-tier bridge instance.
#[derive(Clone)]
pub struct ChartHandle {
    inner: ChartInner,
}

#[derive(Clone)]
enum ChartInner {
    Engine(sys::Chart),
    Bridge { id: f64 },
}

impl ChartHandle {
    /// Create a pure OpenAlgo / Lightweight chart directly on a container DOM element.
    pub fn create(
        container: &web_sys::HtmlElement,
        options: Option<&ChartOptions>,
    ) -> Result<Self, ChartError> {
        let opts_js = if let Some(opts) = options {
            serde_wasm_bindgen::to_value(opts)
                .map_err(|e| ChartError::Serialization(e.to_string()))?
        } else {
            JsValue::UNDEFINED
        };

        // Try direct create_chart, then fallback to window.createChart
        let chart = sys::create_chart(container, &opts_js)
            .or_else(|_| sys::create_chart_window(container, &opts_js))
            .map_err(ChartError::from)?;

        Ok(Self {
            inner: ChartInner::Engine(chart),
        })
    }

    /// Wrap an existing bridge chart id (from `window.honbaCharts.mount`).
    pub fn from_bridge_id(id: f64) -> Self {
        Self {
            inner: ChartInner::Bridge { id },
        }
    }

    /// Returns the bridge ID if this handle is a bridge-backed chart.
    pub fn bridge_id(&self) -> Option<f64> {
        match self.inner {
            ChartInner::Bridge { id } => Some(id),
            _ => None,
        }
    }

    /// Returns the underlying `sys::Chart` if this handle is an engine-backed chart.
    pub fn as_engine(&self) -> Option<&sys::Chart> {
        match &self.inner {
            ChartInner::Engine(chart) => Some(chart),
            _ => None,
        }
    }

    /// Add a new series to the chart.
    pub fn add_series(
        &self,
        kind: SeriesKind,
        options: Option<&SeriesOptions>,
    ) -> Result<SeriesHandle, ChartError> {
        match &self.inner {
            ChartInner::Engine(chart) => {
                let opts_js = if let Some(opts) = options {
                    serde_wasm_bindgen::to_value(opts)
                        .map_err(|e| ChartError::Serialization(e.to_string()))?
                } else {
                    JsValue::UNDEFINED
                };

                let series_api = chart.add_series(kind.as_str(), &opts_js);
                Ok(SeriesHandle {
                    inner: SeriesInner::Engine(series_api),
                })
            }
            ChartInner::Bridge { id } => {
                let opts_json = if let Some(opts) = options {
                    serde_json::to_string(opts).unwrap_or_default()
                } else {
                    String::new()
                };
                Ok(SeriesHandle {
                    inner: SeriesInner::Bridge {
                        chart_id: *id,
                        kind,
                        style_json: opts_json,
                    },
                })
            }
        }
    }

    /// Auto-scale and fit all visible series content into view.
    pub fn fit_content(&self) {
        match &self.inner {
            ChartInner::Engine(chart) => chart.fit_content(),
            ChartInner::Bridge { .. } => {}
        }
    }

    /// Destroy the chart instance and free backing canvas buffers.
    pub fn destroy(&self) {
        match &self.inner {
            ChartInner::Engine(chart) => chart.destroy(),
            ChartInner::Bridge { id } => {
                let _ = sys::bridge_destroy(*id);
            }
        }
    }

    /// Subscribe to crosshair movement events.
    pub fn subscribe_crosshair_move<F>(&self, callback: F) -> Option<Subscription>
    where
        F: Fn(CrosshairMoveEvent) + 'static,
    {
        match &self.inner {
            ChartInner::Engine(chart) => {
                let closure = Closure::<dyn Fn(JsValue)>::new(move |val: JsValue| {
                    let ev = CrosshairMoveEvent::from_js(&val);
                    callback(ev);
                });
                let unsub = chart.subscribe_crosshair_move(closure.as_ref().unchecked_ref());
                Some(Subscription::new(unsub, Box::new(closure)))
            }
            ChartInner::Bridge { .. } => None,
        }
    }

    /// Subscribe to click events on the chart.
    pub fn subscribe_click<F>(&self, callback: F) -> Option<Subscription>
    where
        F: Fn(String) + 'static,
    {
        match &self.inner {
            ChartInner::Engine(chart) => {
                let closure = Closure::<dyn Fn(JsValue)>::new(move |val: JsValue| {
                    let id = val.as_string().unwrap_or_default();
                    callback(id);
                });
                let unsub = chart.subscribe_click(closure.as_ref().unchecked_ref());
                Some(Subscription::new(unsub, Box::new(closure)))
            }
            ChartInner::Bridge { .. } => None,
        }
    }

    // ------------------------------------------------------------------------
    // Bridge / Terminal Features
    // ------------------------------------------------------------------------

    pub fn add_volume_overlay(&self) {
        if let ChartInner::Bridge { id } = self.inner {
            let _ = sys::bridge_add_volume_overlay(id);
        }
    }

    pub fn add_buy_sell_bubble(&self, quantity: i64, symbol: &str) {
        if let ChartInner::Bridge { id } = self.inner {
            let _ = sys::bridge_add_buy_sell(id, quantity as f64, symbol);
        }
    }

    pub fn set_bubble_quantity(&self, quantity: i64) {
        if let ChartInner::Bridge { id } = self.inner {
            let _ = sys::bridge_set_quantity(id, quantity as f64);
        }
    }

    pub fn set_symbol_context(&self, symbol: &str) {
        if let ChartInner::Bridge { id } = self.inner {
            let _ = sys::bridge_set_symbol_context(id, symbol);
        }
    }

    pub fn set_drawing_tool(&self, tool: &str) -> bool {
        if let ChartInner::Bridge { id } = self.inner {
            crate::drawing::set_drawing_tool(id, tool)
        } else {
            false
        }
    }

    pub fn drawing_tool(&self) -> String {
        if let ChartInner::Bridge { id } = self.inner {
            crate::drawing::drawing_tool(id)
        } else {
            "cursor".to_string()
        }
    }

    pub fn undo_drawing(&self) {
        if let ChartInner::Bridge { id } = self.inner {
            crate::drawing::undo_drawing(id);
        }
    }

    pub fn redo_drawing(&self) {
        if let ChartInner::Bridge { id } = self.inner {
            crate::drawing::redo_drawing(id);
        }
    }

    pub fn clear_drawings(&self) {
        if let ChartInner::Bridge { id } = self.inner {
            crate::drawing::clear_drawings(id);
        }
    }

    pub fn add_indicator(
        &self,
        indicator_id: &str,
        settings: &serde_json::Value,
        instance_id: Option<&str>,
    ) -> Result<LiveIndicator, String> {
        if let ChartInner::Bridge { id } = self.inner {
            crate::indicators::add_indicator(id, indicator_id, settings, instance_id)
        } else {
            Err("Indicators only supported on bridge/widget instances".to_string())
        }
    }

    pub fn remove_indicator(&self, instance_id: &str) {
        if let ChartInner::Bridge { id } = self.inner {
            crate::indicators::remove_indicator(id, instance_id);
        }
    }
}

/// A handle to an active series on a chart.
#[derive(Clone)]
pub struct SeriesHandle {
    inner: SeriesInner,
}

#[derive(Clone)]
enum SeriesInner {
    Engine(sys::SeriesApi),
    Bridge {
        chart_id: f64,
        kind: SeriesKind,
        style_json: String,
    },
}

impl SeriesHandle {
    /// Populate or replace all data bars for this series.
    ///
    /// Serializes Rust `[Bar]` directly into JavaScript structures using `serde_wasm_bindgen`.
    pub fn set_data(&self, bars: &[Bar]) -> Result<(), ChartError> {
        match &self.inner {
            SeriesInner::Engine(series) => {
                let js_bars = serde_wasm_bindgen::to_value(bars)
                    .map_err(|e| ChartError::Serialization(e.to_string()))?;
                series.set_data(&js_bars);
                Ok(())
            }
            SeriesInner::Bridge {
                chart_id,
                kind,
                style_json,
            } => {
                let bars_json = serde_json::to_string(bars)
                    .map_err(|e| ChartError::Serialization(e.to_string()))?;
                let _ = sys::bridge_set_series(*chart_id, kind.as_str(), style_json, &bars_json);
                Ok(())
            }
        }
    }

    /// Populate or replace line data points for single-value series (line, area, histogram).
    pub fn set_line_data(&self, points: &[LineData]) -> Result<(), ChartError> {
        match &self.inner {
            SeriesInner::Engine(series) => {
                let js_data = serde_wasm_bindgen::to_value(points)
                    .map_err(|e| ChartError::Serialization(e.to_string()))?;
                series.set_data(&js_data);
                Ok(())
            }
            SeriesInner::Bridge { .. } => Err(ChartError::Js(
                "Direct line data not supported on bridge series".to_string(),
            )),
        }
    }

    /// Update or append a single bar (e.g. streaming ticks).
    pub fn update(&self, bar: &Bar) -> Result<(), ChartError> {
        match &self.inner {
            SeriesInner::Engine(series) => {
                let js_bar = serde_wasm_bindgen::to_value(bar)
                    .map_err(|e| ChartError::Serialization(e.to_string()))?;
                series.update(&js_bar);
                Ok(())
            }
            SeriesInner::Bridge { chart_id, .. } => {
                let bar_json = serde_json::to_string(bar)
                    .map_err(|e| ChartError::Serialization(e.to_string()))?;
                let _ = sys::bridge_update_last(*chart_id, &bar_json);
                Ok(())
            }
        }
    }

    /// Apply options / styles to this series.
    pub fn apply_options(&self, options: &SeriesOptions) -> Result<(), ChartError> {
        match &self.inner {
            SeriesInner::Engine(series) => {
                let js_opts = serde_wasm_bindgen::to_value(options)
                    .map_err(|e| ChartError::Serialization(e.to_string()))?;
                series.series_apply_options(&js_opts);
                Ok(())
            }
            SeriesInner::Bridge { .. } => Ok(()),
        }
    }

    /// Remove the series from its chart pane.
    pub fn remove(&self) {
        if let SeriesInner::Engine(series) = &self.inner {
            series.remove();
        }
    }
}
