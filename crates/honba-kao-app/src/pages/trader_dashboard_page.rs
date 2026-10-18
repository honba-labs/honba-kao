//! Trader dashboard page. Mirrors `web/src/pages/TraderDashboardPage.tsx`.
//!
//! Resolves the selected trader from the store (set when a trader card is
//! opened) and feeds it, plus its account/status, into the terminal console.

use leptos::prelude::*;

use crate::api;
use crate::components::terminal::TerminalDashboard;
use crate::stores::use_traders_config_store;

#[component]
pub fn TraderDashboardPage() -> impl IntoView {
    let store = use_traders_config_store();
    store.ensure_selection();

    view! {
        {move || {
            let trader = store.selected();
            let status = trader
                .as_ref()
                .map(|t| api::traders::get_status(&t.trader_id))
                .unwrap_or_else(|| crate::data::demo_data::demo_status("all"));
            let account = trader
                .as_ref()
                .map(|t| api::traders::get_account(&t.trader_id))
                .unwrap_or_else(crate::data::demo_data::demo_account);

            view! {
                <TerminalDashboard
                    selected_trader=trader
                    status=Some(status)
                    account=Some(account)
                />
            }
        }}
    }
}
