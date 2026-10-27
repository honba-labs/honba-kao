//! Risk Dashboard page component.
use leptos::prelude::*;

/// Risk Dashboard page component.
#[component]
pub fn RiskPage() -> impl IntoView {
    view! {
        <section class="p-6">
            <h2 class="text-2xl font-bold text-primary mb-4">Risk Dashboard</h2>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Value at Risk (VaR)</h3>
                    <p class="text-sm">95% and 99% VaR calculations with EWMA volatility</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Conditional VaR (CVaR)</h3>
                    <p class="text-sm">Expected shortfall beyond VaR threshold</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">PCA Factor Exposures</h3>
                    <p class="text-sm">Factor risk attribution analysis</p>
                </div>
                <div class="p-4 border rounded elevated">
                    <h3 class="font-semibold text-muted mb-2">Stress Scenarios</h3>
                    <p class="text-sm">2008 GFC, COVID-19, Rate Shock historical replays</p>
                </div>
            </div>
        </section>
    }
}