use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn GreeksHeatmapPage() -> impl IntoView {
    view! {
        <HfPage title="Options Greeks Heatmap" subtitle="Surface map of Delta, Gamma, Vega, and Theta across strikes and tenors">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Volatility surface interpolation and charm/vanna cross-sensitivities."
            </div>
        </HfPage>
    }
}
