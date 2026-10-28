//! Risk & Ops domain (plan.md §2.1 "Risk & Ops") — the correlation,
//! executive cockpit, data-quality and reports screens that round out the
//! domain alongside Risk, OMS and Ops (handled elsewhere). Real data
//! wiring is tracked separately in `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn CorrelationPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Correlation"
            subtitle="Cross-asset and cross-position correlation matrix."
            sections=vec![
                ("Correlation Matrix", "Pairwise correlation across open positions and watchlist symbols."),
                ("Concentration Risk", "Clusters of highly correlated exposures flagged for review."),
            ]
        />
    }
}

#[component]
pub fn CockpitPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Cockpit"
            subtitle="Executive dashboard unifying portfolio, risk and market pulse."
            sections=vec![
                ("Portfolio Snapshot", "Total value, net PnL and exposure summary at a glance."),
                ("Risk Heatmap", "Portfolio-level risk concentration across sector and factor."),
                ("Market Pulse", "Sentiment gauge and intelligence timeline digest."),
            ]
        />
    }
}

#[component]
pub fn DataQualityPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Data Quality"
            subtitle="Feed health, data gaps and provider fallback diagnostics."
            sections=vec![
                ("Feed Health", "Per-provider latency, uptime and tick-gap scoring."),
                ("Fallback Waterfall", "Active provider priority and automatic failover history."),
            ]
        />
    }
}

#[component]
pub fn ReportsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Reports"
            subtitle="Scheduled and on-demand compliance and performance reports."
            sections=vec![
                ("Report Library", "Generated portfolio, risk and compliance reports."),
                ("Schedules", "Recurring report generation schedules and recipients."),
            ]
        />
    }
}
