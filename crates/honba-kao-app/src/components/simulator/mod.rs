//! Simulator results surface — the Leptos port of honba-fe
//! `features/simulator/components/*` (port phases P5, chart portions P6):
//! the ribbon, the journal table, the monthly heatmap, the validation
//! badges and the run-log tail, composed below the chart arena by
//! [`crate::pages::simulator_page`].
//!
//! * [`metrics_ribbon`] — provenance-chipped metric tiles.
//! * [`journal_table`] — filter + sort + CSV export of the journal (W5).
//! * [`monthly_heatmap`] — IST month × year realised matrix.
//! * [`validation_badges`] — trust gates + not-modelled assumptions.
//! * [`run_log`] — poll log + SSE event tail drawer.
//! * [`equity_card`] — realised equity + labelled benchmark overlay (P6).
//! * [`simulator_chart`] — bar-loading container with the source badge
//!   (P6; volume pane / indicator overlays / markers pend the bridge work).
//! * [`replay_controls`] — the replay toolbar (P7).
//! * [`signals_panel`] / [`portfolio_panel`] — the P7 dock tabs.

pub mod equity_card;
pub mod journal_table;
pub mod metrics_ribbon;
pub mod monthly_heatmap;
pub mod portfolio_panel;
pub mod replay_controls;
pub mod run_log;
pub mod signals_panel;
pub mod simulator_chart;
pub mod validation_badges;
