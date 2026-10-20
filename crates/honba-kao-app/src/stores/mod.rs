//! Reactive stores. Mirrors the reference app's `web/src/stores/`: Zustand
//! stores become plain structs of `RwSignal`s provided via `provide_context`.

pub mod screener;
pub mod simulator;
pub mod trading_mode;
pub mod workbench;

#[cfg(target_arch = "wasm32")]
pub use simulator::utc_today;
pub use simulator::SimulatorStore;
pub use trading_mode::{TradingMode, TradingModeStore};
