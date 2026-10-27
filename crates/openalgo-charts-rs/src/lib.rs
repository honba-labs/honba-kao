//! # openalgo-charts
//!
//! Idiomatic Rust bindings and Leptos components for [OpenAlgo Charts](https://github.com/marketcalls/openalgo-charts)
//! (TradingView Lightweight Charts fork).
//!
//! Provides:
//! - Direct, type-safe `wasm-bindgen` JS API declarations for `createChart`, `Chart`, `SeriesApi`, `TimeScale`, and `PriceScale`.
//! - High-performance data serialization with `serde-wasm-bindgen` for OHLCV bars (`Bar`) and line series (`LineData`).
//! - Event subscriptions with RAII cleanup (`Closure::wrap`, `Subscription`).
//! - First-class Leptos 0.7 components:
//!   - `<ChartContainer />`: Lean, reactive container for custom canvas charts.
//!   - `<OpenAlgoChart />`: Full terminal chart component supporting static series and widget mounts.
//! - Sophisticated bridging:
//!   - 502 built-in indicators catalog and live instance management.
//!   - Drawing controller with 11+ drawing tools and undo/redo.
//!   - Volume overlay with automatic secondary scale layout.
//!   - Buy/Sell bubble with quantity sync and click stream.
//!   - Dynamic CSS token theme bridging (`theme_from_skin`).

pub mod chart;
pub mod controller;
pub mod drawing;
pub mod events;
pub mod indicators;
pub mod sys;
pub mod theme;
pub mod types;

#[cfg(feature = "leptos")]
pub mod leptos;

pub use chart::{ChartError, ChartHandle, SeriesHandle};
pub use controller::ChartController;
pub use drawing::DrawingTool;
pub use events::{CrosshairMoveEvent, OrderClickEvent, Subscription};
pub use indicators::{
    add_indicator, chart_indicators, indicator_catalog, register_custom_indicator_picker,
    remove_indicator, IndicatorDef, IndicatorInput, LiveIndicator, SelectOption,
};
pub use theme::{css_var, is_dark_mode, resolve_color, theme_from_skin};
pub use types::{
    Bar, ChartOptions, ChartSeries, ChartTheme, GridOptions, LayoutOptions, LineData,
    PriceScaleOptions, SeriesKind, SeriesOptions, TimeScaleOptions,
};

#[cfg(feature = "leptos")]
pub use leptos::{ChartContainer, OpenAlgoChart};

pub mod prelude {
    pub use crate::chart::{ChartError, ChartHandle, SeriesHandle};
    pub use crate::controller::ChartController;
    pub use crate::drawing::DrawingTool;
    pub use crate::events::{CrosshairMoveEvent, OrderClickEvent, Subscription};
    pub use crate::indicators::{IndicatorDef, IndicatorInput, LiveIndicator, SelectOption};
    pub use crate::theme::{css_var, theme_from_skin};
    pub use crate::types::{
        Bar, ChartOptions, ChartSeries, ChartTheme, LineData, SeriesKind, SeriesOptions,
    };

    #[cfg(feature = "leptos")]
    pub use crate::leptos::{ChartContainer, OpenAlgoChart};
}
