//! Derivatives & F&O (`/fno`) — mock-data screens (Phase 3).
//!
//! * [`option_chain_page`] — Option Chain (3.1) with the Strike Summary Bar (3.2).
//! * [`strategy_builder_page`] — Multi-Leg Options Strategy Builder (3.3).
//! * [`oi_page`], [`greeks_heatmap_page`], [`futures_page`], [`flow_page`] —
//!   Derivatives Analytics (3.4a).

mod fmt;

pub mod flow_page;
pub mod futures_page;
pub mod greeks_heatmap_page;
pub mod oi_page;
pub mod option_chain_page;
pub mod strategy_builder_page;

pub use flow_page::FlowPage;
pub use futures_page::FuturesPage;
pub use greeks_heatmap_page::GreeksHeatmapPage;
pub use oi_page::OiPage;
pub use option_chain_page::OptionChainPage;
pub use strategy_builder_page::StrategyBuilderPage;
