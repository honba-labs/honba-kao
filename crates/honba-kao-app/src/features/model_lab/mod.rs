//! Model Lab (Phase 5.4)
//! /backtesting/model-lab — experiment versioning registry, versioned run reports,
//! leaderboard, and model comparison view.

use leptos::prelude::*;

/// Model Lab entry point page.
/// Shows the experiment registry, versioned run reports, and model comparison UI.
#[component]
pub fn ModelLabPage() -> impl IntoView {
    view! {
        <h1 class="text-2xl font-bold text-primary mb-4">Model Lab</h1>
        <p class="text-muted-foreground mb-6">
            "Model experiment registry — under construction. Coming soon: versioned runs,
            leaderboard, and model governance workflows."
        </p>
    }
}