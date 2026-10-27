//! Backtesting Deck (Phase 5)
//! /backtesting — 16+ canonical strategy templates, execution slippage models,
//! equity/drawdown curves, monthly PnL heatmaps, Monte Carlo bootstrap (1,000+ paths),
//! Walk-Forward Optimization (WFO) timelines, statistical robustness badges (DSR, PBO),
//! 3D visualization panels, and strategy export.

pub mod backtesting_page;

use leptos::prelude::*;

/// Backtesting entry point page.
/// Displays the backtesting deck ribbon with strategy templates,
/// runner controls, and analytics panels.
#[component]
pub fn BacktestingPage() -> impl IntoView {
    view! {
        <h1 class="text-2xl font-bold text-primary mb-4">Backtesting Deck</h1>
        <p class="text-muted-foreground mb-6">
            "Backtesting workstation — under construction. Coming soon: strategy templates,
            Monte Carlo simulation, and statistical robustness reports."
        </p>
    }
}
