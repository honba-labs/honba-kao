//! Screener — filterable instrument table, heatmap and screen presets.
//!
//! * [`components`] — filter bar, table, heatmap, dialogs and drawers.
//! * [`stores`] — `columns` (static registries), `filter_engine` (pure
//!   filtering/sorting/CSV) and `screener_store` (the reactive state).
//! * [`data`] — the bundled instrument universe and the index registry.

pub mod components;
pub mod data;
pub mod screener_page;
pub mod stores;

pub use screener_page::ScreenerPage;
