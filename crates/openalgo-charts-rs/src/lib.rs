//! `openalgo-charts-rs` — Standalone financial charting crate.
//!
//! Canvas/WebGL rendering, indicators overlay, drawing rail,
//! and reactive Leptos chart wrapper components.

pub mod canvas;
pub mod chart;
pub mod curves;
pub mod depth;
pub mod drawing;
pub mod events;
pub mod indicators;
pub mod theme;
pub mod types;

#[cfg(feature = "leptos")]
pub mod leptos;

pub use canvas::{CanvasEngine, ChartId, SeriesKind, SeriesStyle, StreamingBarChart, StreamingCandleChart};
pub use chart::ChartController;
pub use curves::{SpreadBand, TermStructure, YieldCurve, YieldPoint};
pub use depth::{DepthLadder, DepthLevel, OrderDispatch};
pub use drawing::{DrawingItem, DrawingTool};
pub use events::{CrosshairMoveEvent, VisibleTimeRangeChangedEvent};
pub use indicators::ChartIndicatorOverlay;
pub use theme::ChartTheme;
pub use types::{CandleData, TimeSeries, TimeSeriesPoint, ViewportState};

#[cfg(feature = "leptos")]
pub use self::leptos::OpenalgoChart;
