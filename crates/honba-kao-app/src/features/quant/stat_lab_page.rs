use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn StatLabPage() -> impl IntoView {
    view! {
        <HfPage title="Statistical Arbitrage Lab" subtitle="Stationarity tests, autocorrelation analysis, and Hurst exponents">
            <div class="p-4 rounded border border-border-soft bg-surface-base text-xs text-text-muted">
                "Augmented Dickey-Fuller (ADF) statistics, variance ratio tests, and Ornstein-Uhlenbeck parameter fits."
            </div>
        </HfPage>
    }
}
