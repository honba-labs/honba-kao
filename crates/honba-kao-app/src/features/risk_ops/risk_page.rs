use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn RiskPage() -> impl IntoView {
    let primary = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base">
            <h3 class="text-sm font-semibold text-text-primary mb-2">"VaR / CVaR Analytics"</h3>
            <div class="text-xs text-text-muted">
                "Parametric and historical 95%/99% VaR, covariance matrix decomposition, and tail risk simulation."
            </div>
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Stress Scenarios"</h3>
            <span class="text-xs text-text-muted">"PCA decomposition and tail risk simulation."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Institutional Risk Analytics"
            subtitle="Value-at-Risk (VaR), Expected Shortfall (CVaR), and stress scenario testing"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
