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
use crate::stores::workbench::{OrderSide, WorkbenchStore};

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

#[component]
pub fn WorkbenchPage() -> impl IntoView {
    let store = WorkbenchStore::new();
    store.provide();

    // Demo seeds live until the backend answers — `fetch_state` overwrites
    // the slices it can reach and leaves the rest in place offline.
    store.fetch_state();

    // The chart bubble's clicks (the bridge dispatches `honba-order-click`
    // from the engine's subscribeClick) open the order modal — labs'
    // subscribeClick wiring, re-homed across the bridge.
    let handler =
        Closure::<dyn FnMut(wasm_bindgen::JsValue)>::new(move |event: wasm_bindgen::JsValue| {
            let Ok(event) = event.dyn_into::<web_sys::CustomEvent>() else {
                return;
            };
            let detail = event.detail();
            let field = |detail: &wasm_bindgen::JsValue, name: &str| -> String {
                js_sys::Reflect::get(detail, &wasm_bindgen::JsValue::from(name))
                    .ok()
                    .and_then(|value| value.as_string())
                    .unwrap_or_default()
            };
            let number = |detail: &wasm_bindgen::JsValue, name: &str| -> f64 {
                js_sys::Reflect::get(detail, &wasm_bindgen::JsValue::from(name))
                    .ok()
                    .and_then(|value| value.as_f64())
                    .unwrap_or_default()
            };
            let action = field(&detail, "action");
            let symbol = field(&detail, "symbol");
            let quantity = number(&detail, "qty").round().max(1.0) as i64;

            match action.as_str() {
                "sell" => {
                    store.open_order_modal(
                        Some(&symbol),
                        Some(OrderSide::Sell),
                        None,
                        Some(quantity),
                    );
                }
                _ => {
                    // `:buy` and `:qty` both lean buy, like the labs terminal.
                    store.open_order_modal(
                        Some(&symbol),
                        Some(OrderSide::Buy),
                        None,
                        Some(quantity),
                    );
                }
            }
        });
    if let Some(window) = web_sys::window() {
        let _ = window.add_event_listener_with_callback(
            "honba-order-click",
            handler.as_ref().unchecked_ref::<js_sys::Function>(),
        );
        handler.forget(); // lives for the whole session; safer than dangling removes
    }

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
