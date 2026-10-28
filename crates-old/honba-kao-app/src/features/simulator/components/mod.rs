//! Simulator results surface — the Leptos port of honba-fe
//! `features/simulator/components/*` (port phases P5, chart portions P6):
//! the ribbon, the journal table, the monthly heatmap, the validation
//! badges and the run-log tail, composed below the chart arena by
//! [`crate::features::simulator::simulator_page`].
//!
//! * [`metrics_ribbon`] — provenance-chipped metric tiles.
//! * [`journal_table`] — filter + sort + CSV export of the journal (W5).
//! * [`monthly_heatmap`] — IST month × year realised matrix.
//! * [`validation_badges`] — trust gates + not-modelled assumptions.
//! * [`run_log`] — poll log + SSE event tail drawer.
//! * [`equity_card`] — realised equity + labelled benchmark overlay (P6).
//! * [`simulator_chart`] — bar-loading container with the source badge
//!   (P6; volume pane / indicator overlays / markers pend the bridge work).
//! * [`replay_controls`] — the replay transport, as the compact
//!   [`replay_controls::ReplayCluster`] in the command bar's center zone.
//! * [`signals_panel`] / [`portfolio_panel`] — the P7 dock tabs.
//! * [`command_bar`] — the Leptos port of fe's `<header
//!   className="sim-command-bar">` (strategy/symbol/timeframe/date-range
//!   pickers, the replay cluster, the live P&L + confluence chips, the
//!   Parameters toggle and the Strategy Studio link).
//! * [`bottom_dock`] — the Leptos port of fe's `<footer
//!   className="sim-bottom-dock">`, fe's exact six tabs (`SimTab::ALL`).
//! * [`kpi_cards`] — the overview tab's four institutional KPI cards.
//! * [`report_panel`] — the `report` dock tab (fe's `results-panel.tsx`).
//! * [`params_drawer`] — the collapsible parameters drawer (fe's
//!   `<aside className="sim-params-drawer">`).

pub mod bottom_dock;
pub mod command_bar;
pub mod equity_card;
pub mod journal_table;
pub mod kpi_cards;
pub mod metrics_ribbon;
pub mod monthly_heatmap;
pub mod params_drawer;
pub mod portfolio_panel;
pub mod replay_controls;
pub mod report_panel;
pub mod run_log;
pub mod signals_panel;
pub mod simulator_chart;
pub mod validation_badges;
