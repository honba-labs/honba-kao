//! Chart Workstation (Phase 4.1) at `/equity/chart-workstation`.
//!
//! Independent of `features/workbench`: its own layout preset, pane state and
//! snapshot store. Charts mount through the shared openalgo-charts bridge.

mod chart_workstation_page;
mod openscript_panel;

pub use chart_workstation_page::ChartWorkstationPage;
pub use openscript_panel::OpenScriptPanel;
