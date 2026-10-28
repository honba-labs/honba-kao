//! Mission Control Home primitives.
//!
//! Faithful Leptos 0.7 CSR port of OpenTerminalUI's home components:
//! - `LiveClockStrip`: Multi-timezone live clocks
//! - `MarketHeatStrip`: Real-time index and commodity price strip
//! - `SparklineCell`: SVG sparkline with gradient fill and tooltip
//! - `MetricCard`: Institutional KPI card with delta badge, details and sparkline
//! - `SystemHealthBar`: Multi-pill feed health, WebSocket status and latency
//! - `QuickNavGrid`: 4-column keyboard-navigable grid of workspace shortcuts

pub mod clock_strip;
pub mod market_heat_strip;
pub mod metric_card;
pub mod quick_nav_grid;
pub mod sparkline_cell;
pub mod system_health_bar;

pub use clock_strip::*;
pub use market_heat_strip::*;
pub use metric_card::*;
pub use quick_nav_grid::*;
pub use sparkline_cell::*;
pub use system_health_bar::*;
