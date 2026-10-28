//! Canvas engine handle: a live OpenAlgo Charts chart bound to a host canvas.
//!
//! `CanvasEngine` owns a chart id returned by `window.honbaCharts.mountStatic`
//! and exposes `set_series` / `update_last` / `destroy`. A chart is "mounted"
//! once the host supplies a canvas element; until then the engine holds the
//! pending configuration and applies it on mount.

use crate::canvas::bridge;
use crate::types::CandleData;
use wasm_bindgen::JsValue;

/// Series kind handed to the OpenAlgo Charts engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SeriesKind {
    #[default]
    Candlestick,
    Bar,
    Line,
    Area,
    Histogram,
}

impl SeriesKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SeriesKind::Candlestick => "candlestick",
            SeriesKind::Bar => "bar",
            SeriesKind::Line => "line",
            SeriesKind::Area => "area",
            SeriesKind::Histogram => "histogram",
        }
    }
}

/// A series style bag, serialised to JSON for the bridge.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct SeriesStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_width: Option<f64>,
}

impl SeriesStyle {
    pub fn candlestick(up: &str, down: &str) -> Self {
        Self {
            up_color: Some(up.to_string()),
            down_color: Some(down.to_string()),
            ..Default::default()
        }
    }
    pub fn line(color: &str) -> Self {
        Self {
            color: Some(color.to_string()),
            line_width: Some(2.0),
            ..Default::default()
        }
    }
    pub fn histogram(color: &str) -> Self {
        Self {
            color: Some(color.to_string()),
            ..Default::default()
        }
    }
}

/// Opaque chart id returned by the engine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartId(pub f64);

/// A mounted canvas chart engine.
pub struct CanvasEngine {
    element: web_sys::HtmlCanvasElement,
    id: Option<ChartId>,
    kind: SeriesKind,
    style: SeriesStyle,
    theme: Option<crate::theme::ChartTheme>,
}

impl CanvasEngine {
    /// Prepare an engine bound to `element`. The chart is not mounted until
    /// [`Self::mount`] is called (or [`Self::apply_pending`] after mount).
    pub fn new(
        element: web_sys::HtmlCanvasElement,
        kind: SeriesKind,
        style: SeriesStyle,
    ) -> Self {
        Self {
            element,
            id: None,
            kind,
            style,
            theme: None,
        }
    }

    pub fn with_theme(mut self, theme: crate::theme::ChartTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Mount the chart on the bound canvas via `window.honbaCharts.mountStatic`.
    /// Returns the chart id.
    pub fn mount(&mut self) -> Result<ChartId, JsValue> {
        let id = bridge::mount_static(&self.element)?;
        self.id = Some(ChartId(id));
        Ok(ChartId(id))
    }

    /// Chart id once mounted, `None` otherwise.
    pub fn id(&self) -> Option<ChartId> {
        self.id
    }

    /// Replace the whole series (candles/bars/line/area). Serialises `data` to
    /// JSON and pushes it through `window.honbaCharts.setSeries`.
    pub fn set_series(&self, data: &[CandleData]) -> Result<bool, JsValue> {
        let id = self.id.ok_or_else(|| JsValue::from_str("chart not mounted"))?;
        let style_json = serde_json::to_string(&self.style)
            .map_err(|e| JsValue::from_str(&format!("style serialise: {e}")))?;
        let bars_json = serde_json::to_string(data)
            .map_err(|e| JsValue::from_str(&format!("bars serialise: {e}")))?;
        bridge::set_series(id.0, self.kind.as_str(), &style_json, &bars_json)
    }

    /// Update or append the last bar via `window.honbaCharts.updateLast` — the
    /// efficient path for live streaming ticks.
    pub fn update_last(&self, candle: &CandleData) -> Result<bool, JsValue> {
        let id = self.id.ok_or_else(|| JsValue::from_str("chart not mounted"))?;
        let bar_json = serde_json::to_string(candle)
            .map_err(|e| JsValue::from_str(&format!("bar serialise: {e}")))?;
        bridge::update_last(id.0, &bar_json)
    }

    /// Attach a volume histogram overlay to the primary pane.
    pub fn add_volume_overlay(&self) -> Result<bool, JsValue> {
        let id = self.id.ok_or_else(|| JsValue::from_str("chart not mounted"))?;
        bridge::add_volume_overlay(id.0)
    }

    /// Destroy the chart and release its canvases.
    pub fn destroy(self) -> Result<bool, JsValue> {
        if let Some(id) = self.id {
            bridge::destroy(id.0)
        } else {
            Ok(false)
        }
    }
}

impl std::fmt::Debug for CanvasEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasEngine")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .finish()
    }
}
