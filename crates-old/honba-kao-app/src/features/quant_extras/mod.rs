//! Quant domain (plan.md §2.1 "Quant") — the algorithm framework, alpha
//! zoo, research autopilot and strategy export screens that round out the
//! domain alongside Backtesting, Model Lab, Model Governance, Stat Lab,
//! Pair Trading, Strategy Studio and Simulator (handled elsewhere). Real
//! implementations are tracked separately in `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn AlgoFrameworkPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Algorithm Framework"
            subtitle="Reusable building blocks for composing systematic strategies."
            sections=vec![
                ("Building Blocks", "Catalog of signal, sizing and execution components available to strategies."),
                ("Composition Graph", "Visual dependency graph of how blocks wire into a complete strategy."),
            ]
        />
    }
}

#[component]
pub fn AlphaZooPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Alpha Zoo"
            subtitle="Catalog of candidate alpha factors and signal generators."
            sections=vec![
                ("Factor Catalog", "Browsable library of candidate alpha signals with summary statistics."),
                ("Signal Correlation", "Pairwise correlation matrix across the active factor set."),
            ]
        />
    }
}

#[component]
pub fn ResearchAutopilotPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Research Autopilot"
            subtitle="Autonomous research loop proposing and testing new strategy variants."
            sections=vec![
                ("Run Queue", "Autopilot-proposed strategy variants queued for backtest evaluation."),
                ("Findings Log", "Discoveries and rejected hypotheses with the reasoning behind each."),
            ]
        />
    }
}

#[component]
pub fn StrategyExportPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Strategy Export"
            subtitle="Export a validated strategy to a deployable live-trading artifact."
            sections=vec![
                ("Export Targets", "Package a strategy for `honba-ya` live deployment or a shareable bundle."),
                ("Pre-flight Checklist", "Validation gates a strategy must pass before export is enabled."),
            ]
        />
    }
}
