//! Reactive stores (≈ Zustand slices). Mirrors `web/src/stores/`.

pub mod traders_config_store;
pub mod traders_modal_store;

pub use traders_config_store::{TradersConfigStore, use_traders_config_store};
pub use traders_modal_store::{TraderModalMode, TradersModalStore, use_traders_modal_store};
