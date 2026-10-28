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

pub mod bridge;
pub mod engine;
pub mod candles;
pub mod bars;

pub use bridge::{mount_static, set_series, update_last, add_volume_overlay, destroy, refresh_theme_all};
pub use engine::{CanvasEngine, SeriesKind, SeriesStyle, ChartId};
pub use candles::StreamingCandleChart;
pub use bars::StreamingBarChart;
