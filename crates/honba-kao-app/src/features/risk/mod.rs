//! Risk Dashboard — VaR/CVaR, EWMA vol, PCA factor exposures, and crisis stress scenarios.
use leptos::prelude::*;

/// Risk Dashboard page component.
#[component]
pub fn RiskPage() -> impl IntoView {
    view! {
        <h2 class="text-2xl font-bold text-primary mt-4">Risk Dashboard</h2>
        <p class="text-muted mb-6">
            "Risk dashboard under construction — VaR, CVaR, EWMA volatility, PCA factor exposures,"
            " and crisis stress scenario visualizations will appear here."
        </p>
    }
}
