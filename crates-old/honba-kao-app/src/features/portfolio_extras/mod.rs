//! Portfolio & Execution domain (plan.md §2.1 "Portfolio") — the paper
//! trading, trade journal and alerts screens that round out the domain
//! alongside Portfolio, Portfolio Lab, Optimizer, Position Sizer and Shadow
//! Account (handled elsewhere). Real data wiring is tracked separately in
//! `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn PaperPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Paper Trading"
            subtitle="Simulated order routing against live or delayed quotes, no capital at risk."
            sections=vec![
                ("Paper Blotter", "Simulated orders and fills booked against the paper account."),
                ("Paper Positions", "Open simulated positions with mark-to-market PnL."),
            ]
        />
    }
}

#[component]
pub fn JournalPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Journal"
            subtitle="Trade journal — rationale, screenshots and post-trade review."
            sections=vec![
                ("Journal Entries", "Per-trade notes: setup, rationale, and outcome review."),
                ("Tag Analytics", "Win-rate and expectancy broken down by journal tag."),
            ]
        />
    }
}

#[component]
pub fn AlertsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Alerts"
            subtitle="Price, indicator and event-based alert management."
            sections=vec![
                ("Active Alerts", "Configured price, indicator and event alerts with trigger conditions."),
                ("Alert History", "Log of alerts that fired, with timestamp and triggering value."),
            ]
        />
    }
}
