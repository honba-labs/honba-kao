//! Workbench — the trading cockpit. Charting workspace, order tickets and
//! the per-symbol depth/position drawer, composing the body ported from the
//! labs frontend's `workbench-app.tsx`.
//!
//! The page owns the [`WorkbenchStore`] context; everything under the page
//! (toolbar, chart grid, ticket, drawer, modal) reads it via
//! `WorkbenchStore::take()`.

use leptos::prelude::*;

use crate::components::workbench::chart_grid::MultiChartGrid;
use crate::components::workbench::indicator_picker::IndicatorPickerDialog;
use crate::components::workbench::order_modal::WorkbenchOrderModal;
use crate::components::workbench::order_ticket::OrderTicketPanel;
use crate::components::workbench::symbol_search::SymbolSearchModal;
use crate::components::workbench::toolbar::WorkbenchToolbar;
use crate::components::workbench::trading_drawer::TradingDrawer;
use crate::stores::workbench::WorkbenchStore;

#[component]
pub fn WorkbenchPage() -> impl IntoView {
    let store = WorkbenchStore::new();
    store.provide();

    view! {
        <div class="wb-view">
            <WorkbenchToolbar />

            <div class="wb-body">
                <MultiChartGrid />
                <Show when=move || store.ticket_open.get()>
                    <OrderTicketPanel />
                </Show>
            </div>

            <TradingDrawer />
            <WorkbenchOrderModal />
            <SymbolSearchModal />
            <IndicatorPickerDialog />
        </div>
    }
}
