//! Reactive stores. Mirrors the reference app's `web/src/stores/`: Zustand
//! stores become plain structs of `RwSignal`s provided via `provide_context`.
//!
//! The screener is split into `columns` (static registries),
//! `filter_engine` (pure filtering/sorting/CSV) and `screener_store` (the
//! reactive state). The workbench is split into `layout_store`,
//! `order_draft_store` and `book_store`, composed by `workbench_store`.

pub mod book_store;
pub mod columns;
pub mod filter_engine;
pub mod layout_store;
pub mod order_draft_store;
pub mod screener_store;
pub mod simulator;
pub mod trading_mode;
pub mod workbench_store;

#[cfg(target_arch = "wasm32")]
pub use simulator::utc_today;
pub use simulator::SimulatorStore;
pub use trading_mode::{TradingMode, TradingModeStore};
