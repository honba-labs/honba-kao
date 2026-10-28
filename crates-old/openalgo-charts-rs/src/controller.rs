//! Framework-agnostic chart controller for headless and multi-framework integration (Leptos, Dioxus, Yew, etc.).

use wasm_bindgen::JsCast;

use crate::chart::{ChartError, ChartHandle, SeriesHandle};
use crate::events::{CrosshairMoveEvent, Subscription};
use crate::types::{Bar, ChartOptions, SeriesKind, SeriesOptions};

/// Headless chart lifecycle and state controller.
///
/// Designed to be embedded into any reactive framework state (e.g. Dioxus `use_signal`,
/// Leptos `StoredValue`, or Yew component state).
#[derive(Default)]
pub struct ChartController {
    chart: Option<ChartHandle>,
    primary_series: Option<SeriesHandle>,
    crosshair_sub: Option<Subscription>,
    click_sub: Option<Subscription>,
}

impl ChartController {
    pub const fn new() -> Self {
        Self {
            chart: None,
            primary_series: None,
            crosshair_sub: None,
            click_sub: None,
        }
    }

    /// Mount a new chart onto the specified DOM element.
    pub fn mount(
        &mut self,
        container: &web_sys::HtmlElement,
        options: Option<&ChartOptions>,
        kind: SeriesKind,
        series_options: Option<&SeriesOptions>,
        initial_bars: &[Bar],
    ) -> Result<&ChartHandle, ChartError> {
        // Destroy any existing chart first to prevent leaks
        self.destroy();

        let handle = ChartHandle::create(container, options)?;
        let series = handle.add_series(kind, series_options)?;
        let _ = series.set_data(initial_bars);
        handle.fit_content();

        self.primary_series = Some(series);
        self.chart = Some(handle);

        Ok(self.chart.as_ref().unwrap())
    }

    /// Mount by DOM element ID (convenient for Dioxus / Yew where elements have string IDs).
    pub fn mount_by_id(
        &mut self,
        element_id: &str,
        options: Option<&ChartOptions>,
        kind: SeriesKind,
        series_options: Option<&SeriesOptions>,
        initial_bars: &[Bar],
    ) -> Result<&ChartHandle, ChartError> {
        let window = web_sys::window().ok_or_else(|| ChartError::Js("No window".to_string()))?;
        let document = window
            .document()
            .ok_or_else(|| ChartError::Js("No document".to_string()))?;
        let el = document
            .get_element_by_id(element_id)
            .ok_or_else(|| ChartError::Js(format!("Element #{element_id} not found")))?;
        let html_el: web_sys::HtmlElement = el
            .dyn_into()
            .map_err(|_| ChartError::Js("Element is not an HtmlElement".to_string()))?;

        self.mount(&html_el, options, kind, series_options, initial_bars)
    }

    /// Returns whether a chart is currently mounted and alive.
    pub fn is_mounted(&self) -> bool {
        self.chart.is_some()
    }

    /// Access the underlying `ChartHandle`.
    pub fn handle(&self) -> Option<&ChartHandle> {
        self.chart.as_ref()
    }

    /// Access the primary `SeriesHandle`.
    pub fn primary_series(&self) -> Option<&SeriesHandle> {
        self.primary_series.as_ref()
    }

    /// Update all data bars for the primary series.
    pub fn set_data(&self, bars: &[Bar]) -> Result<(), ChartError> {
        if let Some(series) = &self.primary_series {
            series.set_data(bars)
        } else {
            Err(ChartError::Destroyed)
        }
    }

    /// Append or update the most recent bar.
    pub fn update_last(&self, bar: &Bar) -> Result<(), ChartError> {
        if let Some(series) = &self.primary_series {
            series.update(bar)
        } else {
            Err(ChartError::Destroyed)
        }
    }

    /// Auto-fit all data content into the visible canvas.
    pub fn fit_content(&self) {
        if let Some(chart) = &self.chart {
            chart.fit_content();
        }
    }

    /// Attach a crosshair movement listener.
    pub fn on_crosshair<F>(&mut self, callback: F)
    where
        F: Fn(CrosshairMoveEvent) + 'static,
    {
        if let Some(chart) = &self.chart {
            self.crosshair_sub = chart.subscribe_crosshair_move(callback);
        }
    }

    /// Attach an element click listener.
    pub fn on_click<F>(&mut self, callback: F)
    where
        F: Fn(String) + 'static,
    {
        if let Some(chart) = &self.chart {
            self.click_sub = chart.subscribe_click(callback);
        }
    }

    /// Destroy the chart and clean up all subscriptions (guaranteed leak-free).
    pub fn destroy(&mut self) {
        if let Some(sub) = self.crosshair_sub.take() {
            sub.unsubscribe();
        }
        if let Some(sub) = self.click_sub.take() {
            sub.unsubscribe();
        }
        self.primary_series = None;
        if let Some(chart) = self.chart.take() {
            chart.destroy();
        }
    }
}

impl Drop for ChartController {
    fn drop(&mut self) {
        self.destroy();
    }
}
