//! Chart Workstation (Phase 4.1) at `/equity/chart-workstation`.
//!
//! Independent of `features/workbench`: its own layout preset, pane state and
//! snapshot store. Charts mount through the shared openalgo-charts bridge.

mod chart_workstation_page;

pub use chart_workstation_page::ChartWorkstationPage;
