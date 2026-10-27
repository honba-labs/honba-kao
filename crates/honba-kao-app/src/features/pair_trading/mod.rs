//! Pair Trading Lab (Phase 5.6)
//! /equity/pair-trading — cointegration testing, spread analysis, and signal generation.

pub mod pair_trading_page;

use leptos::prelude::*;

/// Pair Trading entry point page.
/// Shows spread analysis, cointegration signals, and trade execution tools.
#[component]
pub fn PairTradingPage() -> impl IntoView {
    view! {
        <h1 class="text-2xl font-bold text-primary mb-4">Pair Trading</h1>
        <p class="text-muted-foreground mb-6">
            "Pair trading analytics — under construction. Coming soon: cointegration testing,
            spread analysis, and trade signal generation."
        </p>
    }
}
