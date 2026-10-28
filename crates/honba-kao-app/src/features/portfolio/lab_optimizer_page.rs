use honba_kao_ui_core::templates::{HfPage, HfTwoColumnSplitLayout};
use leptos::prelude::*;

#[component]
pub fn LabOptimizerPage() -> impl IntoView {
    let primary = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base">
            <h3 class="text-sm font-semibold text-text-primary mb-2">"Factor Optimizer"</h3>
            <div class="text-xs text-text-muted">
                "Mean-variance efficient frontier, factor tilts, and risk parity allocation."
            </div>
        </div>
    }
    .into_any();

    let rail = view! {
        <div class="p-4 rounded border border-border-soft bg-surface-base flex flex-col gap-3">
            <h3 class="text-xs font-semibold text-text-primary mb-1">"Allocation Controls"</h3>
            <span class="text-xs text-text-muted">"HRP, Black-Litterman, covariance shrinkage."</span>
        </div>
    }
    .into_any();

    view! {
        <HfPage
            title="Portfolio Lab & Factor Optimizer"
            subtitle="Mean-variance efficient frontier, factor tilts, and risk parity allocation"
        >
            <HfTwoColumnSplitLayout primary=primary rail=rail />
        </HfPage>
    }
}
