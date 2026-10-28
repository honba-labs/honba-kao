//! Markets & Overview domain (plan.md §2.1 "Markets") — structural
//! placeholders for the market-wide monitoring, watchlist, and macro screens
//! that round out the nav rail's first domain. Each page is a thin wrapper
//! around [`FeaturePlaceholderPage`]; real data wiring is tracked separately
//! in `plan.md`'s phase checklist.

use leptos::prelude::*;

use crate::shared::components::FeaturePlaceholderPage;

#[component]
pub fn MarketPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Market Overview"
            subtitle="Broad market indices, breadth and session summary."
            sections=vec![
                ("Index Strip", "NIFTY 50, SENSEX, BANKNIFTY and sector indices with live change%."),
                ("Market Breadth", "Advance/decline counts, new highs/lows, and up/down volume ratio."),
                ("Session Summary", "Opening range, VWAP drift and closing auction notes."),
                ("Top Movers", "Largest gainers, losers and volume shockers across the exchange."),
            ]
        />
    }
}

#[component]
pub fn MonitoringPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Monitoring"
            subtitle="Cross-symbol alert and condition monitoring board."
            sections=vec![
                ("Active Monitors", "Symbols currently being watched against price, volume or indicator triggers."),
                ("Trigger Log", "Chronological record of monitor conditions that fired this session."),
                ("Feed Health", "Per-symbol tick latency and staleness indicators."),
            ]
        />
    }
}

#[component]
pub fn WatchlistPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Watchlist"
            subtitle="User-curated symbol lists with live quotes."
            sections=vec![
                ("My Lists", "Named watchlists with drag-to-reorder and quick add/remove."),
                ("Live Quotes", "LTP, change% and volume for every symbol in the active list."),
                ("Quick Actions", "Jump to Security Hub, Chart Workstation or Option Chain from any row."),
            ]
        />
    }
}

#[component]
pub fn HotlistsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Hotlists"
            subtitle="Algorithmically ranked lists of unusual market activity."
            sections=vec![
                ("Volume Shockers", "Symbols trading at multiples of their average daily volume."),
                ("Breakouts", "Symbols crossing multi-period highs or lows intraday."),
                ("Gap Movers", "Symbols with the largest open-to-current gaps."),
            ]
        />
    }
}

#[component]
pub fn HeatmapPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Heatmap"
            subtitle="Sector and index-constituent performance heatmap."
            sections=vec![
                ("Sector Tiles", "Market-cap weighted tiles coloured by intraday change%."),
                ("Drill-down", "Click a sector tile to expand into its constituent heatmap."),
            ]
        />
    }
}

#[component]
pub fn NewsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="News"
            subtitle="Aggregated market and corporate news feed."
            sections=vec![
                ("Headline Stream", "Real-time headlines tagged by symbol and sentiment."),
                ("Filters", "Filter by symbol, sector, source or sentiment polarity."),
            ]
        />
    }
}

#[component]
pub fn EconomicsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Economics"
            subtitle="Macro calendar and economic indicator dashboard."
            sections=vec![
                ("Economic Calendar", "Scheduled releases (CPI, GDP, policy rate decisions) with consensus vs. actual."),
                ("Indicator Trends", "Historical trend charts for key macro series."),
            ]
        />
    }
}

#[component]
pub fn CommoditiesPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Commodities"
            subtitle="Commodity futures quotes and curves."
            sections=vec![
                ("Quote Board", "Crude, gold, silver and agricultural futures with change%."),
                ("Forward Curves", "Contango/backwardation curves per contract."),
            ]
        />
    }
}

#[component]
pub fn ForexPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Forex"
            subtitle="Major and INR cross-rate quote board."
            sections=vec![
                ("Quote Board", "USDINR, EURINR and major cross pairs with live spreads."),
                ("Volatility", "Implied volatility and historical range per pair."),
            ]
        />
    }
}

#[component]
pub fn CryptoPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Crypto"
            subtitle="Digital asset quote board."
            sections=vec![
                ("Quote Board", "Top digital assets by market cap with 24h change%."),
                ("Market Cap Ranking", "Sortable ranking table by cap, volume and dominance."),
            ]
        />
    }
}

#[component]
pub fn BondsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Bonds"
            subtitle="Government and corporate bond quote board."
            sections=vec![
                ("Benchmark Yields", "10Y, 5Y and short-end government security yields."),
                ("Credit Spreads", "Corporate-bond spreads over the sovereign curve."),
            ]
        />
    }
}

#[component]
pub fn YieldCurvePage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Yield Curve"
            subtitle="Sovereign yield curve visualization and history."
            sections=vec![
                ("Current Curve", "Spot yield curve across maturities."),
                ("Curve History", "Overlay of the curve at prior dates to visualize shifts."),
            ]
        />
    }
}

#[component]
pub fn EtfPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="ETF Analytics"
            subtitle="Exchange-traded fund screening and NAV analytics."
            sections=vec![
                ("ETF Screener", "Filter ETFs by asset class, expense ratio and tracking error."),
                ("NAV vs. Price", "Premium/discount to NAV over time."),
            ]
        />
    }
}

#[component]
pub fn MutualFundsPage() -> impl IntoView {
    view! {
        <FeaturePlaceholderPage
            title="Mutual Funds"
            subtitle="Mutual fund screening and performance analytics."
            sections=vec![
                ("Fund Screener", "Filter by category, AUM, expense ratio and risk rating."),
                ("Performance", "Rolling returns vs. benchmark and category average."),
            ]
        />
    }
}
