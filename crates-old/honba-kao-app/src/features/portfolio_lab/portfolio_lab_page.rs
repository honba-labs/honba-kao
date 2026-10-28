use leptos::prelude::*;

/// Portfolio Lab page — optimization and strategy blending.
#[component]
pub fn PortfolioLabPageContent() -> impl IntoView {
    view! {
        <section class="prose max-w-5xl">
            <h2 class="text-xl font-semibold mb-4">Optimization Models</h2>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
                <div>
                    <h3 class="font-medium mb-2">Mean-Variance</h3>
                    <p class="text-xs text-muted-foreground">
                        "Efficient frontier optimization with risk constraints."
                    </p>
                </div>
                <div>
                    <h3 class="font-medium mb-2">Black-Litterman</h3>
                    <p class="text-xs text-muted-foreground">
                        "Incorporate subjective views on asset expected returns."
                    </p>
                </div>
                <div>
                    <h3 class="font-medium mb-2">Risk Parity</h3>
                    <p class="text-xs text-muted-foreground">
                        "Equal risk contribution across all assets."
                    </p>
                </div>
            </div>
        </section>

        <section class="mt-8">
            <h2 class="text-xl font-semibold mb-4">Strategy Blending</h2>
            <p class="text-muted-foreground mb-4">
                "Blend multiple strategies into a cohesive portfolio with custom weights."
            </p>
        </section>
    }
}