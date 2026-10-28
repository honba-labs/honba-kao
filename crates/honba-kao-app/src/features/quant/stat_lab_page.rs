use honba_kao_ui_core::templates::HfMosaicWorkspace;
use leptos::prelude::*;

#[component]
pub fn StatLabPage() -> impl IntoView {
    view! {
        <HfMosaicWorkspace>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-xs font-semibold text-text-primary mb-2">"Stationarity Tests"</h3>
                <div class="text-xs text-text-muted">
                    "Augmented Dickey-Fuller (ADF) statistics and variance ratio tests."
                </div>
            </div>
            <div class="col-span-12 laptop:col-span-6 p-4 rounded border border-border-soft bg-surface-base">
                <h3 class="text-xs font-semibold text-text-primary mb-2">"Hurst Exponent"</h3>
                <div class="text-xs text-text-muted">
                    "Ornstein-Uhlenbeck parameter fits and mean-reversion half-life estimates."
                </div>
            </div>
        </HfMosaicWorkspace>
    }
}
