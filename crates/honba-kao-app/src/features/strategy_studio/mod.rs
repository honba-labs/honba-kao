//! Strategy studio — the strategy library and the strategy-graph editor.
//!
//! * [`kernel`] — the pure strategy-graph kernel (validation, hashing,
//!   diffing) and the code⇄graph Python codegen; no DOM, no signals. It runs
//!   over the graph shape the simulator kernel carries
//!   ([`crate::features::simulator::kernel::selection::StrategyGraphShape`]).
//! * [`components`] — the strategy library and grid risk panel.

pub mod components;
pub mod kernel;
pub mod strategy_studio_page;

pub use strategy_studio_page::StrategyStudioPage;
