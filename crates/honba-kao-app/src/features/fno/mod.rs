//! Derivatives & F&O (`/fno`) — mock-data screens (Phase 3).
//!
//! * [`option_chain_page`] — Option Chain (3.1) with the Strike Summary Bar (3.2).
//! * [`strategy_builder_page`] — Multi-Leg Options Strategy Builder (3.3).

pub mod option_chain_page;
pub mod strategy_builder_page;

pub use option_chain_page::OptionChainPage;
pub use strategy_builder_page::StrategyBuilderPage;
