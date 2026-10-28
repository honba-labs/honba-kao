use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn OmsPage() -> impl IntoView {
    view! {
        <HfPage title="Order Management System (OMS) Compliance" subtitle="Exchange limits, fat-finger protection, and circuit filter status">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Pre-trade risk checks, price execution bands, maximum single-order notional caps, and throttle controls."
            </div>
        </HfPage>
    }
}
