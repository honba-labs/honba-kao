//! Active traders list. Mirrors `web/src/components/trader/TradersList.tsx`.
//!
//! Selection + navigation live here (via the store and `use_navigate`) so the
//! component needs no callback prop.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::ui::{
    ButtonVariant, HfBadge, HfButton, HfPLNumber, HfPanel, HfPanelHeader, TraderState,
};
use crate::router::paths::build_dashboard_path;
use crate::stores::use_traders_config_store;
use crate::types::TraderInfo;
use crate::utils::percent;

fn state_of(trader: &TraderInfo) -> TraderState {
    if trader.is_running {
        TraderState::Running
    } else {
        TraderState::Stopped
    }
}

#[component]
pub fn TradersList(#[prop(optional)] traders: Option<Vec<TraderInfo>>) -> impl IntoView {
    let store = use_traders_config_store();
    let navigate = use_navigate();
    let rows = move || traders.clone().unwrap_or_else(|| store.traders.get());

    view! {
        <HfPanel>
            <HfPanelHeader title="Active traders" counter=rows().len()>
                <HfButton variant=ButtonVariant::Green small=true>"+ New"</HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content trader-cards">
                {move || {
                    let navigate = navigate.clone();
                    rows()
                        .into_iter()
                        .map(|trader| {
                            let trader_id = trader.trader_id.clone();
                            let nav = navigate.clone();
                            let open = move |_ev: web_sys::MouseEvent| {
                                store.select(trader_id.clone());
                                let slug = store.find_by_id(&trader_id).map(|t| t.slug());
                                nav(&build_dashboard_path(slug.as_deref()), Default::default());
                            };

                            view! {
                                <div class="trader-card">
                                    <div class="trader-card__head">
                                        <strong>{trader.trader_name.clone()}</strong>
                                        <HfBadge state=state_of(&trader) />
                                    </div>
                                    <div class="trader-card__meta">
                                        {format!("{} · {}", trader.ai_model, trader.exchange)}
                                    </div>
                                    <div class="trader-card__pnl">
                                        <span>"Session P&L"</span>
                                        <HfPLNumber value=trader.pnl is_currency=true />
                                    </div>
                                    <div class="trader-card__meta">
                                        "Win rate "{percent(trader.win_rate)}
                                    </div>
                                    <div class="trader-card__actions">
                                        <button class="hfui-button small" on:click=open>
                                            "Open"
                                        </button>
                                        <HfButton variant=ButtonVariant::Red small=true>"Stop"</HfButton>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </HfPanel>
    }
}
