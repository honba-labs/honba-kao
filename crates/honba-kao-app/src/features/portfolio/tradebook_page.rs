use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn TradebookPage() -> impl IntoView {
    view! {
        <HfPage title="Tradebook & Execution Journal" subtitle="Audited execution records, turnover metrics, and tax analytics">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "FIFO/LIFO pairing execution reports, exchange fee breakdown, and transaction tax calculations."
            </div>
        </HfPage>
    }
}
