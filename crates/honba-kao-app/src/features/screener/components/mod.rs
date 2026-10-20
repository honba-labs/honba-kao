//! Screener page components — the Leptos port of the labs screener
//! (`filter bar · table · heatmap · modals · drawers`). Layout:
//!
//! * [`filter_bar`] — top bar (asset-class pills, screen picker triggers,
//!   quick presets, search, live badge, export CSV).
//! * [`screener_table`] — structural table + per-column header menus +
//!   pagination.
//! * [`heatmap`] — sector-grouped market-cap tile matrix.
//! * [`filters_modal`] / [`column_modal`] / [`open_screen_modal`] — the
//!   three configuration dialogs.
//! * [`drawers`] — index drawer + symbol detail drawer.

pub mod column_modal;
pub mod drawers;
pub mod filter_bar;
pub mod filters_modal;
pub mod heatmap;
pub mod open_screen_modal;
pub mod screener_table;
