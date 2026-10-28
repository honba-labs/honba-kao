//! Simulator — strategy backtests over the server run API plus the local
//! bar-by-bar replay engine.
//!
//! * [`kernel`] — the pure simulator kernel (controls planning, run/journal/
//!   log folds, chart data, manifest, replay engine); no DOM, no signals.
//! * [`stores`] — `controls_store`, `run_store` and `replay_store`, composed
//!   by the `simulator_store` coordinator.
//! * [`components`] — the results surface below the chart arena.

pub mod components;
pub mod kernel;
pub mod simulator_page;
pub mod stores;

pub use simulator_page::SimulatorPage;
