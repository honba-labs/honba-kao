//! Workbench — the trading cockpit body ported from the labs frontend's
//! `src/apps/workbench-app.tsx` (React) to Leptos:
//!
//! ```text
//!   workbench-app.tsx            ->  workbench_page.rs (composition)
//!   core/workbench/store.ts      ->  stores/workbench_store.rs
//!   workbench-toolbar.tsx        ->  toolbar.rs
//!   multi-chart-grid.tsx         ->  chart_grid.rs (grid + pane card)
//!   chart-pane-card/openalgo     ->  pane_chart.rs (pane header + canvas)
//!   order-ticket.tsx             ->  order_ticket.rs
//!   drawing rail                 ->  drawing_rail.rs (engine DrawingController)
//!   order-modal.tsx              ->  order_modal.rs
//!   trading-drawer.tsx           ->  trading_drawer.rs
//!   market-depth.tsx             ->  market_depth.rs
//!   lib/chart/layouts.tsx        ->  layouts.rs
//! ```
//!
//! Styling lives in `style/workbench.css` on the token contract
//! (`--background-color`, `--shade-*`, `--primary-*`, `--pnl-gain/loss`), per
//! the plan's zero-inlined-styles rule.

pub mod chart_grid;
pub mod drawing_rail;
pub mod indicator_picker;
pub mod layouts;
pub mod market_depth;
pub mod number_input;
pub mod order_modal;
pub mod order_ticket;
pub mod pane_chart;
pub mod symbol_search;
pub mod toolbar;
pub mod trading_drawer;
