use honba_kao_ui_core::templates::HfMosaicWorkspace;
use leptos::prelude::*;

#[component]
pub fn ModelLabPage() -> impl IntoView {
    view! {
        <HfMosaicWorkspace>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-xs font-semibold text-text-primary mb-2">"Alpha Factor Lab"</h3>
                <div class="text-xs text-text-muted">
                    "LightGBM/XGBoost factor predictions, SHAP value breakdowns, and rank information coefficients (IC)."
                </div>
            </div>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-xs font-semibold text-text-primary mb-2">"Cross-Validation"</h3>
                <div class="text-xs text-text-muted">
                    "Out-of-sample performance, walk-forward stability, and regime-aware parameter fits."
                </div>
            </div>
        </HfMosaicWorkspace>
    }
}
