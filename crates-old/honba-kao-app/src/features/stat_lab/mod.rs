//! Statistical & Pair Trading Labs (Phase 5.6)
//! /equity/stat-lab and /equity/pair-trading — cointegration testing, ADF unit root
//! tests, half-life mean-reversion, and z-score spread tracking.

pub mod stat_lab_page;

use leptos::prelude::*;

/// Statistical Lab entry point page.
/// Shows cointegration tests, ADF unit root tests, and half-life analysis.
#[component]
pub fn StatLabPage() -> impl IntoView {
    view! {
        <h1 class="text-2xl font-bold text-primary mb-4">Statistical Lab</h1>
        <p class="text-muted-foreground mb-6">
            "Statistical analysis tools — under construction. Coming soon: cointegration testing,
            ADF unit root tests, half-life mean-reversion, and z-score spread tracking."
        </p>
    }
}
