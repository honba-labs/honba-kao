//! App-wide reactive stores. Mirrors the reference app's `web/src/stores/`:
//! Zustand stores become plain structs of `RwSignal`s provided via
//! `provide_context`. Feature-owned stores live in each
//! `features/<name>/stores/`.

pub mod trading_mode;

pub use trading_mode::{TradingMode, TradingModeStore};
