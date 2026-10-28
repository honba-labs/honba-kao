//! Canvas engine handle: a live OpenAlgo Charts chart bound to a host canvas.
//!
//! `CanvasEngine` owns a chart id returned by `window.honbaCharts.mountStatic`
//! and exposes `set_series` / `update_last` / `destroy`. A chart is "mounted"
//! once the host supplies a canvas element; until then the engine holds the
//! pending `kind`/`style`/`theme` configuration. [`CanvasEngine::mount`] folds
//! any pending theme into the active style (see [`CanvasEngine::apply_pending`])
//! before mounting, and guards against leaking a previously mounted chart if
//! called again on the same engine.

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
///
/// Integer-typed (u64) per the zero-floats invariant: chart ids are opaque
/// handles returned by `window.honbaCharts.mountStatic`, never fractional
/// quantities. The wasm/JS bridge marshals them as f64 at the boundary and we
/// convert here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChartId(pub u64);

/// A mounted canvas chart engine.
pub struct CanvasEngine {
    element: web_sys::HtmlCanvasElement,
    id: Option<ChartId>,
    kind: SeriesKind,
    style: SeriesStyle,
    theme: Option<crate::theme::ChartTheme>,
}

impl CanvasEngine {
    /// Prepare an engine bound to `element`, using `kind`/`style` as the
    /// initial series configuration. The chart is not mounted until
    /// [`Self::mount`] is called.
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

    /// Set the pending theme. Immediately folds it into the active
    /// [`SeriesStyle`] via [`Self::apply_pending`], so [`Self::style`]
    /// reflects it right away even before [`Self::mount`] runs.
    pub fn with_theme(mut self, theme: crate::theme::ChartTheme) -> Self {
        self.theme = Some(theme);
        self.apply_pending();
        self
    }

    /// Fold the pending theme (set via [`Self::with_theme`]) into the active
    /// series style, so the colors used by the next [`Self::set_series`] /
    /// [`Self::update_last`] call reflect it. This touches no bridge/JS
    /// state — it is a pure, idempotent, Rust-side update — so it is safe to
    /// call at any time; [`Self::mount`] calls it automatically.
    pub fn apply_pending(&mut self) {
        if let Some(theme) = self.theme.as_ref() {
            self.style.up_color = Some(theme.up_color.clone());
            self.style.down_color = Some(theme.down_color.clone());
            self.style.color = Some(theme.up_color.clone());
        }
    }

    /// Mount the chart on the bound canvas via `window.honbaCharts.mountStatic`.
    /// Returns the chart id. If this engine already has a mounted chart
    /// (i.e. `mount` is called a second time on the same engine), the
    /// existing chart is destroyed first so it is not leaked.
    pub fn mount(&mut self) -> Result<ChartId, JsValue> {
        if let Some(existing) = self.id.take() {
            let _ = bridge::destroy(existing.0 as f64);
        }
        self.apply_pending();
        let id = bridge::mount_static(&self.element)?;
        self.id = Some(ChartId(id as u64));
        Ok(ChartId(id as u64))
    }

    /// Chart id once mounted, `None` otherwise.
    pub fn id(&self) -> Option<ChartId> {
        self.id
    }

    /// The active series style — reflects any theme folded in via
    /// [`Self::apply_pending`].
    pub fn style(&self) -> &SeriesStyle {
        &self.style
    }

    /// Replace the whole series (candles/bars/line/area). Serialises `data` to
    /// JSON and pushes it through `window.honbaCharts.setSeries`.
    pub fn set_series(&self, data: &[CandleData]) -> Result<bool, JsValue> {
        let id = self.id.ok_or_else(|| JsValue::from_str("chart not mounted"))?;
        let style_json = serde_json::to_string(&self.style)
            .map_err(|e| JsValue::from_str(&format!("style serialise: {e}")))?;
        let bars_json = serde_json::to_string(data)
            .map_err(|e| JsValue::from_str(&format!("bars serialise: {e}")))?;
        bridge::set_series(id.0 as f64, self.kind.as_str(), &style_json, &bars_json)
    }

    /// Update or append the last bar via `window.honbaCharts.updateLast` — the
    /// efficient path for live streaming ticks.
    pub fn update_last(&self, candle: &CandleData) -> Result<bool, JsValue> {
        let id = self.id.ok_or_else(|| JsValue::from_str("chart not mounted"))?;
        let bar_json = serde_json::to_string(candle)
            .map_err(|e| JsValue::from_str(&format!("bar serialise: {e}")))?;
        bridge::update_last(id.0 as f64, &bar_json)
    }

    /// Attach a volume histogram overlay to the primary pane.
    pub fn add_volume_overlay(&self) -> Result<bool, JsValue> {
        let id = self.id.ok_or_else(|| JsValue::from_str("chart not mounted"))?;
        bridge::add_volume_overlay(id.0 as f64)
    }

    /// Destroy the chart and release its canvases.
    pub fn destroy(mut self) -> Result<bool, JsValue> {
        if let Some(id) = self.id.take() {
            bridge::destroy(id.0 as f64)
        } else {
            Ok(false)
        }
    }
}

impl Drop for CanvasEngine {
    /// Release the mounted chart on `window.honbaCharts` when the engine is
    /// dropped without an explicit [`Self::destroy`], so a chart handle is
    /// never leaked (issue #70). Degrades gracefully off-wasm32.
    fn drop(&mut self) {
        if let Some(id) = self.id {
            let _ = bridge::destroy(id.0 as f64);
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
