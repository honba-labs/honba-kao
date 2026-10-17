//! 本場・顔 honba-kao — Leptos SPA root.
//! High-performance trading terminal modeled after bfx-hf-ui-core.

pub mod components;
pub mod theme;
pub mod ui;

use components::{
    DecisionLogPanel, NavRoute, Navbar, OrdersTablePanel, PositionsTablePanel, StatusBar,
    ThemeStudioModal, TradersOverviewPanel,
};
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use theme::ThemeContext;
use wasm_bindgen::prelude::wasm_bindgen;

#[component]
pub fn App() -> impl IntoView {
    // Initialize ThemeContext and provide to component tree
    let theme_ctx = ThemeContext::new();
    provide_context(theme_ctx);

    let active_route = RwSignal::new(NavRoute::Dashboard);
    let is_live_mode = RwSignal::new(false);
    let is_palette_open = RwSignal::new(false);

    let on_open_palette = Callback::new(move |_| {
        is_palette_open.set(true);
    });

    view! {
        <div class="hfui-app">
            // Pinned Top Navigation Bar
            <Navbar
                active_route=active_route
                is_live_mode=is_live_mode
                on_open_palette=on_open_palette
            />

            // Terminal Multi-Panel Workspace Grid
            <main class="hfui-workspace">
                // Left Column: Traders Overview & Controls
                <div class="hfui-workspace__col">
                    <TradersOverviewPanel />
                </div>

                // Middle Column: Live Positions & Active Orders
                <div class="hfui-workspace__col">
                    <PositionsTablePanel />
                    <OrdersTablePanel />
                </div>

                // Right Column: Decision Journal & Risk Gate Audit
                <div class="hfui-workspace__col">
                    <DecisionLogPanel />
                </div>
            </main>

            // Pinned Bottom Status Bar
            <StatusBar />

            // 256-Color Palette Studio & Theme Inspector Modal
            <ThemeStudioModal is_open=is_palette_open />
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    mount_to_body(App);
}
