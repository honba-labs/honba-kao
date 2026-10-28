use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn ShadowAccountPage() -> impl IntoView {
    view! {
        <HfPage title="Shadow Account Reconciliation" subtitle="Cross-broker position synchronization and ledger auditing">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Real-time cash and collateral reconciliation between local oms state and exchange/broker clearing reports."
            </div>
        </HfPage>
    }
}
