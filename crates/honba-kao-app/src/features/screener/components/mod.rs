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
//!
//! Milestone 3 ("Hero Header, Preset Shelf & Query Configuration Deck") adds
//! the deck rendered above the filter bar:
//! * [`hero_deck`] — header ribbon + horizontally scrolling preset shelf.
//! * [`screen_setup`] — tab bar, active-filter summary chips, and the
//!   Saved/Public tab content.
//! * [`query_builder`] — raw query editor (`QueryBar`) + GUI filter row
//!   builder (`QueryBuilder`).
//! * [`custom_formula`] — freeform formula input + metric token cheat sheet.
//! * [`multi_market_scan`] — cross-universe scan checklist.

pub mod column_modal;
pub mod custom_formula;
pub mod drawers;
pub mod factor_heatmap;
pub mod filter_bar;
pub mod filters_modal;
pub mod heatmap;
pub mod hero_deck;
pub mod multi_market_scan;
pub mod open_screen_modal;
pub mod query_builder;
pub mod screen_setup;
pub mod screener_table;
