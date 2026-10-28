use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn RiskPage() -> impl IntoView {
    view! {
        <HfPage title="Institutional Risk Analytics" subtitle="Value-at-Risk (VaR), Expected Shortfall (CVaR), and stress scenario testing">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Parametric and historical 95%/99% VaR, covariance matrix decomposition, and tail risk simulation."
            </div>
        </HfPage>
    }
}
