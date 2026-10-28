//! Research domain (plan.md §2.1 "Research") — structural placeholders for
//! the factor, insider, dividend/earnings and research-library screens that
//! round out the nav rail's second domain alongside Security Hub, Screener
//! and the Research Dossiers (handled elsewhere). Real data wiring is
//! tracked separately in `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn FactorsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Factors"
            subtitle="Factor exposure screening — value, quality, momentum, size and low-volatility."
            sections=vec![
                ("Factor Screener", "Rank the universe by a single factor or a blended multi-factor score."),
                ("Factor Exposure", "Per-symbol exposure breakdown across the standard factor set."),
            ]
        />
    }
}

#[component]
pub fn InsiderPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Insider"
            subtitle="Insider and bulk/block deal activity tracker."
            sections=vec![
                ("Insider Trades", "Promoter and insider buy/sell disclosures with transaction size."),
                ("Bulk & Block Deals", "Exchange-reported bulk and block deal tape."),
            ]
        />
    }
}

#[component]
pub fn DividendsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Dividends"
            subtitle="Dividend calendar, yield screener and payout history."
            sections=vec![
                ("Dividend Calendar", "Upcoming ex-date, record-date and payment-date schedule."),
                ("Yield Screener", "Rank the universe by trailing or forward dividend yield."),
            ]
        />
    }
}

#[component]
pub fn EarningsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Earnings"
            subtitle="Earnings calendar and QoQ/YoY result scorecards."
            sections=vec![
                ("Earnings Calendar", "Upcoming and recent result dates across the universe."),
                ("Result Scorecards", "Revenue, EBITDA and EPS vs. estimate, QoQ and YoY."),
            ]
        />
    }
}

#[component]
pub fn IntelligenceTimelinePage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Intelligence Timeline"
            subtitle="Real-time corporate events and concall summary feed."
            sections=vec![
                ("Event Timeline", "Chronological feed of filings, corporate actions and concall highlights."),
                ("Summaries", "AI-generated summaries of management commentary and guidance."),
            ]
        />
    }
}

#[component]
pub fn ResearchLibraryPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Research Library"
            subtitle="Saved research notes, reports and annotated charts."
            sections=vec![
                ("Saved Notes", "User-authored research notes linked to symbols and strategies."),
                ("Report Archive", "Previously generated dossiers and analyst reports."),
            ]
        />
    }
}

#[component]
pub fn SectorRotationPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Sector Rotation"
            subtitle="Relative sector performance and rotation signal tracker."
            sections=vec![
                ("Rotation Quadrant", "Relative-strength vs. momentum quadrant chart per sector."),
                ("Rotation Timeline", "Historical sector leadership changes over trailing periods."),
            ]
        />
    }
}

#[component]
pub fn RelativeStrengthPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Relative Strength"
            subtitle="Symbol-vs-benchmark and symbol-vs-sector relative strength ranking."
            sections=vec![
                ("RS Ranking", "Universe ranked by relative strength against the chosen benchmark."),
                ("RS Line Chart", "Relative-strength line overlay for the selected symbol."),
            ]
        />
    }
}
