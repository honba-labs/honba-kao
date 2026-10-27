//! Portfolio Lab & Optimizer (Phase 5.5)
//! /equity/portfolio/lab and /backtesting/portfolio-optimizer — strategy blending,
//! Mean-Variance Efficient Frontier, Black-Litterman, and Risk Parity optimization.

use leptos::prelude::*;

/// Portfolio Lab entry point page.
/// Shows strategy blending, optimization models, and allocation results.
#[component]
pub fn PortfolioLabPage() -> impl IntoView {
    view! {
        <h1 class="text-2xl font-bold text-primary mb-4">Portfolio Lab</h1>
        <p class="text-muted-foreground mb-6">
            "Portfolio optimization and strategy blending — under construction. Coming soon:
            Mean-Variance, Black-Litterman, and Risk Parity optimization."
        </p>
    }
}