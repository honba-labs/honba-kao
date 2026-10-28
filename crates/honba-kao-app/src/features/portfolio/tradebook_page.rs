use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn TradebookPage() -> impl IntoView {
    let primary = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base">
            <h3 class="text-sm font-semibold text-text-primary mb-2">"Audited Execution Journal"</h3>
            <div class="text-xs text-text-muted">
                "FIFO/LIFO pairing execution reports, exchange fee breakdown, and transaction tax calculations."
            </div>
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Turnover & Tax"</h3>
            <span class="text-xs text-text-muted">"Section 43(1) turnover and STT computation ready."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Tradebook & Execution Journal"
            subtitle="Audited execution records, turnover metrics, and tax analytics"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
