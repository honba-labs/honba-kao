//! Tier: canvas engine bindings — bindings to the OpenAlgo Charts / Lightweight
//! Charts canvas engine exposed by `window.honbaCharts` (vendored under
//! `vendor/openalgo-charts/`).
//!
//! Provides:
//! - `canvas::bridge` — raw `window.honbaCharts.*` calls (mount, set series,
//!   update last, destroy).
//! - `canvas::engine` — `CanvasEngine`, `SeriesKind`, `SeriesStyle`, `ChartId`.
//! - `canvas::candles` — `StreamingCandleChart` (streaming candlestick renderer).
//! - `canvas::bars` — `StreamingBarChart` (streaming OHLC bar renderer).

pub mod bars;
pub mod bridge;
pub mod candles;
pub mod engine;

pub use bars::StreamingBarChart;
pub use bridge::{
    add_volume_overlay, destroy, mount_static, refresh_theme_all, set_series, update_last,
};
pub use candles::StreamingCandleChart;
pub use engine::{CanvasEngine, ChartId, SeriesKind, SeriesStyle};
