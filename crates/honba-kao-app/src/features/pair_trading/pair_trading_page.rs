use leptos::prelude::*;

/// Pair Trading page — spread analysis and signal generation.
#[component]
pub fn PairTradingPageContent() -> impl IntoView {
    view! {
        <section class="prose max-w-5xl">
            <h2 class="text-xl font-semibold mb-4">Symbol Pairs</h2>
            <p class="text-muted-foreground mb-4">
                "List of cointegrated pairs with current spread statistics."
            </p>
        </section>

        <section class="mt-8">
            <h2 class="text-xl font-semibold mb-4">Spread Analysis</h2>
            <p class="text-muted-foreground mb-4">
                "Real-time spread monitoring with z-score and half-life metrics."
            </p>
        </section>

        <section class="mt-8">
            <h2 class="text-xl font-semibold mb-4">Trade Signals</h2>
            <p class="text-muted-foreground mb-4">
                "Generate entry/exit signals based on statistical thresholds."
            </p>
        </section>
    }
}
