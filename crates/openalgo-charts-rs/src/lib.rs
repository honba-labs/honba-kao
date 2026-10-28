//! `openalgo-charts-rs` — Standalone financial charting crate.
//!
//! Canvas/WebGL rendering, indicators overlay, drawing rail,
//! and reactive Leptos chart wrapper components.

pub mod chart;
pub mod drawing;
pub mod events;
pub mod indicators;
pub mod theme;
pub mod types;

#[cfg(feature = "leptos")]
pub mod leptos;

pub use chart::ChartController;
pub use drawing::{DrawingItem, DrawingTool};
pub use events::{CrosshairMoveEvent, VisibleTimeRangeChangedEvent};
pub use indicators::ChartIndicatorOverlay;
pub use theme::ChartTheme;
pub use types::{CandleData, TimeSeries, TimeSeriesPoint, ViewportState};

#[cfg(feature = "leptos")]
pub use self::leptos::OpenalgoChart;
