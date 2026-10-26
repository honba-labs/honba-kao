//! Multi-Timeframe Analysis (Phase 4.3) at `/equity/mta`.
//!
//! Independent of `features/workbench` and `features/chart_workstation`: its
//! own preset table and pane wiring. Charts mount through the shared
//! openalgo-charts bridge.

mod mta_page;

pub use mta_page::MtaPage;
