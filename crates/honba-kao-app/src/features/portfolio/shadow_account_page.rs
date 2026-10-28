use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn ShadowAccountPage() -> impl IntoView {
    let primary = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base">
            <h3 class="text-sm font-semibold text-text-primary mb-2">"Cross-Broker Reconciliation"</h3>
            <div class="text-xs text-text-muted">
                "Real-time cash and collateral reconciliation between local OMS state and exchange/broker clearing reports."
            </div>
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Connected Brokers"</h3>
            <span class="text-xs text-text-muted">"Zerodha Kite, Angel One, Paper Wallet."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Shadow Account Reconciliation"
            subtitle="Cross-broker position synchronization and ledger auditing"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
