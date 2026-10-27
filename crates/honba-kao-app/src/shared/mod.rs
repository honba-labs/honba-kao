//! Cross-feature code: everything used by more than one feature slice or by
//! the app shell.
//!
//! * [`api`] — namespaced client modules over the typed transport.
//! * [`components`] — shared presentational components and charts.
//! * [`constants`], [`types`], [`i18n`] — branding/auth constants, domain
//!   types, translations.
//! * [`contexts`] — auth, language and confirm-dialog providers.
//! * [`data`] — shared sample data.
//! * [`filings`] — real exact-match citation search over a small checked-in
//!   sample filings corpus (not a production RAG pipeline).
//! * [`hooks`] — reusable reactive helpers.
//! * [`stores`] — app-wide reactive stores (trading mode).
//! * [`theme`] — palette tokens and the theme store.
//! * [`utils`] — formatting, indicator and text helpers.

pub mod api;
pub mod command;
pub mod components;
pub mod constants;
pub mod contexts;
pub mod data;
pub mod filings;
pub mod hooks;
pub mod i18n;
pub mod stores;
pub mod stream;
pub mod theme;
pub mod types;
pub mod utils;
