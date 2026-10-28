use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn ModelLabPage() -> impl IntoView {
    view! {
        <HfPage title="Quantitative Model Lab" subtitle="Machine learning alpha factors, feature importance, and cross-validation">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "LightGBM/XGBoost factor predictions, SHAP value breakdowns, and rank information coefficients (IC)."
            </div>
        </HfPage>
    }
}
