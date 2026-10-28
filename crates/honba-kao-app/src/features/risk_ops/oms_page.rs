use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn OmsPage() -> impl IntoView {
    let primary = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base">
            <h3 class="text-sm font-semibold text-text-primary mb-2">"Pre-Trade Compliance"</h3>
            <div class="text-xs text-text-muted">
                "Exchange limits, fat-finger protection, and circuit filter status."
            </div>
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Circuit Filters"</h3>
            <span class="text-xs text-text-muted">"Max single-order notional caps and throttle controls."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Order Management System (OMS) Compliance"
            subtitle="Exchange limits, fat-finger protection, and circuit filter status"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
