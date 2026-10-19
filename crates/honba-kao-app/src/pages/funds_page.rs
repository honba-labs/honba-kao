//! Funds — balances and margin across the account.

use leptos::prelude::*;

use crate::api;
use crate::components::ui::{HfPanel, HfPanelHeader, PageHeader};
use crate::utils::currency;

#[component]
pub fn FundsPage() -> impl IntoView {
    let account = api::wallet::get_account();
    let day_class = if account.day_pnl >= 0.0 {
        "positive"
    } else {
        "negative"
    };

    view! {
        <div class="console">
            <PageHeader title="Funds" subtitle="Cash, margin and buying power" />

            <HfPanel>
                <HfPanelHeader title="Account" />
                <div class="hfui-panel__content">
                    <div class="summary-row">
                        <span>"Available cash"</span>
                        <strong class="mono">{currency(account.available_cash)}</strong>
                    </div>
                    <div class="summary-row">
                        <span>"Buying power"</span>
                        <strong class="mono">{currency(account.buying_power)}</strong>
                    </div>
                    <div class="summary-row">
                        <span>"Total value"</span>
                        <strong class="mono">{currency(account.total_value)}</strong>
                    </div>
                    <div class="summary-row">
                        <span>"Day P&L"</span>
                        <strong class={format!("mono {day_class}")}>
                            {currency(account.day_pnl)}
                        </strong>
                    </div>
                </div>
            </HfPanel>
        </div>
    }
}
