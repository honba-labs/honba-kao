//! Workbench — the trading cockpit body ported from the labs frontend's
//! `src/apps/workbench-app.tsx` (React) to Leptos:
//!
//! ```text
//!   workbench-app.tsx            ->  pages/workbench_page.rs (composition)
//!   core/workbench/store.ts      ->  stores/workbench.rs
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

use leptos::prelude::*;

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

/// Dismiss a popover menu when the pointer goes down outside `host`.
///
/// The listener is (re)installed while the menu is open and torn down by the
/// effect scope — Leptos' rough equivalent of the React `mousedown`
/// listeners labs attach in `useEffect`.
pub fn dismiss_on_outside_click(open: RwSignal<bool>, host: NodeRef<leptos::html::Div>) {
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        let Some(element) = host.get() else {
            return;
        };
        let handle = window_event_listener_untyped("mousedown", move |event: web_sys::Event| {
            let target = event_target::<web_sys::Element>(&event);
            if !element.contains(Some(&target)) {
                open.set(false);
            }
        });
        on_cleanup(move || handle.remove());
    });
}
