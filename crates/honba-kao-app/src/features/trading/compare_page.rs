use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn ComparePage() -> impl IntoView {
    view! {
        <HfPage title="Relative Performance Comparison" subtitle="Normalized percentage overlay vs Nifty 50, Bank Nifty, and Global indices">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Relative strength, beta vs index, and correlation matrix."
            </div>
        </HfPage>
    }
}
