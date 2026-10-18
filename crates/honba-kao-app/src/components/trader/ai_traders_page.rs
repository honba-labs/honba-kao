//! AI traders page. Mirrors `web/src/components/trader/AITradersPage.tsx`.

use leptos::prelude::*;

use crate::components::trader::TradersList;
use crate::components::ui::{ButtonVariant, HfButton, HfModal, Kpi, PageHeader};
use crate::hooks::use_interval;
use crate::stores::{TraderModalMode, use_traders_config_store, use_traders_modal_store};
use crate::utils::compact_currency;

#[component]
pub fn AITradersPage() -> impl IntoView {
    let store = use_traders_config_store();
    let modal = use_traders_modal_store();
    let tick = use_interval(5_000);

    let total_pnl = move || store.traders.get().iter().map(|t| t.pnl).sum::<f64>();
    let running_count = move || store.traders.get().iter().filter(|t| t.is_running).count();

    view! {
        <div class="console">
            <PageHeader title="AI traders" subtitle="Configure, launch and supervise your agents">
                <HfButton
                    variant=ButtonVariant::Green
                    on_click=Callback::new(move |_| modal.open_create())
                >
                    "+ New trader"
                </HfButton>
            </PageHeader>

            <div class="kpi-grid">
                <Kpi label="Configured">{move || store.traders.get().len().to_string()}</Kpi>
                <Kpi label="Running">{move || running_count().to_string()}</Kpi>
                <Kpi label="Combined P&L">{move || compact_currency(total_pnl())}</Kpi>
                <Kpi label="Refresh">
                    {move || format!("5s · tick {}", tick.get())}
                </Kpi>
            </div>

            <TradersList />

            {move || (modal.mode.get() != TraderModalMode::Closed).then(|| {
                let is_create = modal.mode.get() == TraderModalMode::Create;
                view! {
                    <HfModal
                        title=if is_create { "New trader" } else { "Edit trader" }
                        is_open=true
                        on_close=Callback::new(move |_| modal.close())
                    >
                        <div class="form-grid">
                            <label class="auth-field">
                                <span>"Name"</span>
                                <input placeholder="Momentum v2" />
                            </label>
                            <label class="auth-field">
                                <span>"AI model"</span>
                                <input placeholder="deepseek-v4.1" />
                            </label>
                            <label class="auth-field">
                                <span>"Exchange"</span>
                                <input placeholder="NSE" />
                            </label>
                            <label class="auth-field">
                                <span>"Strategy"</span>
                                <input placeholder="EMA Trend Rider" />
                            </label>
                        </div>
                        <div class="modal-actions">
                            <HfButton
                                variant=ButtonVariant::Ghost
                                on_click=Callback::new(move |_| modal.close())
                            >
                                "Cancel"
                            </HfButton>
                            <HfButton
                                variant=ButtonVariant::Green
                                on_click=Callback::new(move |_| modal.close())
                            >
                                {if is_create { "Create" } else { "Save" }}
                            </HfButton>
                        </div>
                    </HfModal>
                }
            })}
        </div>
    }
}
