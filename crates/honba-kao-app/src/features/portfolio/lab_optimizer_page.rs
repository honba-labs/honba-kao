use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn LabOptimizerPage() -> impl IntoView {
    view! {
        <HfPage title="Portfolio Lab & Factor Optimizer" subtitle="Mean-variance efficient frontier, factor tilts, and risk parity allocation">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Hierarchical Risk Parity (HRP), Black-Litterman views, and covariance shrinkage estimation."
            </div>
        </HfPage>
    }
}
