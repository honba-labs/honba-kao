use leptos::prelude::*;

/// Statistical Lab page — cointegration and unit root tests.
#[component]
pub fn StatLabPageContent() -> impl IntoView {
    view! {
        <section class="prose max-w-5xl">
            <h2 class="text-xl font-semibold mb-4">Cointegration Tests</h2>
            <p class="text-muted-foreground mb-4">
                "Engle-Granger two-step and Johansen test for cointegrated pairs."
            </p>
        </section>

        <section class="mt-8">
            <h2 class="text-xl font-semibold mb-4">ADF Unit Root Test</h2>
            <p class="text-muted-foreground mb-4">
                "Augmented Dickey-Fuller test for stationarity."
            </p>
        </section>

        <section class="mt-8">
            <h2 class="text-xl font-semibold mb-4">Half-Life Mean Reversion</h2>
            <p class="text-muted-foreground mb-4">
                "Ornstein-Uhlenbeck estimation for mean-reversion half-life."
            </p>
        </section>

        <section class="mt-8">
            <h2 class="text-xl font-semibold mb-4">Z-Score Spread Tracking</h2>
            <p class="text-muted-foreground mb-4">
                "Real-time z-score monitoring for pair trading signals."
            </p>
        </section>
    }
}
