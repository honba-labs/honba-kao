//! HomePage — Mission Control Dashboard container.
//!
//! Port of OpenTerminalUI's `HomePage.tsx` (plan-1.md §3 Milestone 3): the
//! full-fidelity mission control landing page that replaces the legacy
//! `/portfolio` post-login redirect. Composes the Milestone 1 UI primitives
//! (`LiveClockStrip`, `MarketHeatStrip`, `MetricCard`, `SystemHealthBar`,
//! `QuickNavGrid`) and the Milestone 2 composite panels
//! (`PortfolioMiniChart`, `IntelWire`).

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use honba_kao_ui_core::{
    HfButton, HfButtonSize, HfButtonVariant,
    LiveClockStrip,
    MarketHeatStrip, MarketHeatStripItem, MarketFlash,
    MetricCard, MetricCardDelta, MetricCardDetail, MetricTone,
    SystemHealthBar, SystemHealthItem, SystemHealthTone,
    QuickNavGrid, QuickNavSection, QuickNavItem,
};

use crate::features::home::components::{
    IntelWire, IntelWireItem, IntelSentimentScore, IntelSentiment,
    PortfolioMiniChart,
};
use crate::features::home::components::portfolio_mini_chart::ChartPoint;
use crate::router::paths::ROUTES;
use crate::shared::contexts::use_auth;

/// Mission Control Dashboard page.
///
/// Layout (plan-1.md §3 3.3):
/// - Header ribbon: kicker/title, metadata badges, action buttons,
///   `LiveClockStrip` in the top right.
/// - `MarketHeatStrip` beneath the header.
/// - 2-column responsive grid: left (1.85fr) Net Liquidation `MetricCard`
///   + `PortfolioMiniChart`; right (1fr) `SystemHealthBar` + `IntelWire`.
/// - Bottom: `QuickNavGrid` launch matrix organised by domain.
#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let navigate = StoredValue::new(use_navigate());

    let go_to = move |route: &'static str| {
        navigate.with_value(|nav| nav(route, Default::default()));
    };

    let user_name = move || auth.display_name();

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 3.2 Market Heat Strip — indices & commodities
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    let heat_items = vec![
        MarketHeatStripItem::new("NIFTY 50", "NIFTY 50", Some(22_450.75), Some(0.85))
            .with_flash(MarketFlash::Up)
            .with_change_label("+0.85%"),
        MarketHeatStripItem::new("SENSEX", "SENSEX", Some(74_120.50), Some(-0.32))
            .with_flash(MarketFlash::Down)
            .with_change_label("-0.32%"),
        MarketHeatStripItem::new("BANKNIFTY", "BANKNIFTY", Some(48_520.00), Some(1.20))
            .with_flash(MarketFlash::Up)
            .with_change_label("+1.20%"),
        MarketHeatStripItem::new("NASDAQ", "NASDAQ", Some(18_750.20), Some(0.45))
            .with_flash(MarketFlash::Up)
            .with_change_label("+0.45%"),
        MarketHeatStripItem::new("S&P 500", "S&P 500", Some(5_420.80), Some(-0.15))
            .with_flash(MarketFlash::Down)
            .with_change_label("-0.15%"),
        MarketHeatStripItem::new("GOLD", "GOLD", Some(2_650.40), Some(0.60))
            .with_flash(MarketFlash::Up)
            .with_change_label("+0.60%"),
        MarketHeatStripItem::new("CRUDE OIL", "CRUDE OIL", Some(78.30), Some(-0.90))
            .with_flash(MarketFlash::Down)
            .with_change_label("-0.90%"),
    ];

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 3.3 Left column — Net Liquidation MetricCard + PortfolioMiniChart
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    let portfolio_points = vec![
        ChartPoint::Labeled {
            label: "Mon".to_string(),
            value: 1_000_000.0,
        },
        ChartPoint::Labeled {
            label: "Tue".to_string(),
            value: 1_023_400.0,
        },
        ChartPoint::Labeled {
            label: "Wed".to_string(),
            value: 1_010_200.0,
        },
        ChartPoint::Labeled {
            label: "Thu".to_string(),
            value: 1_045_600.0,
        },
        ChartPoint::Labeled {
            label: "Fri".to_string(),
            value: 1_078_900.0,
        },
    ];

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 3.3 Right column — SystemHealthBar + IntelWire
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    let health_items = vec![
        SystemHealthItem::new("feed", "Market Feed", "Live", SystemHealthTone::Ok),
        SystemHealthItem::new("websocket", "WebSocket", "Connected", SystemHealthTone::Ok),
        SystemHealthItem::new("session", "Session", "Active", SystemHealthTone::Ok),
        SystemHealthItem::new("latency", "Latency", "42 ms", SystemHealthTone::Ok),
    ];

    let intel_items = vec![
        IntelWireItem::new("n1", "Reuters", "RBI holds repo rate steady at 6.50 %")
            .with_published_at("2026-01-15T09:05:00Z")
            .with_sentiment(IntelSentimentScore::new(IntelSentiment::Neutral, 0.62)),
        IntelWireItem::new("n2", "Bloomberg", "Nifty 50 approaches all-time high")
            .with_published_at("2026-01-15T08:42:00Z")
            .with_sentiment(IntelSentimentScore::new(IntelSentiment::Bullish, 0.78)),
        IntelWireItem::new("n3", "ET Markets", "Bank Nifty sees profit-booking")
            .with_published_at("2026-01-15T08:15:00Z")
            .with_sentiment(IntelSentimentScore::new(IntelSentiment::Bearish, 0.54)),
        IntelWireItem::new("n4", "Reuters", "Crude oil slips on inventory build")
            .with_published_at("2026-01-15T07:50:00Z")
            .with_sentiment(IntelSentimentScore::new(IntelSentiment::Bearish, 0.41)),
        IntelWireItem::new("n5", "CNBC", "IT stocks lead broad market rally")
            .with_published_at("2026-01-15T07:20:00Z")
            .with_sentiment(IntelSentimentScore::new(IntelSentiment::Bullish, 0.69)),
    ];

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 3.3 Bottom — QuickNavGrid launch matrix by domain
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    let sections = vec![
        QuickNavSection::new(
            "markets",
            "Markets",
            vec![
                QuickNavItem::new("equity", "Equity", "[M1]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::MARKET))),
                QuickNavItem::new("watchlist", "Watchlist", "[M2]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::WATCHLIST))),
                QuickNavItem::new("heatmap", "Heatmap", "[M3]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::HEATMAP))),
                QuickNavItem::new("news", "News", "[M4]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::NEWS))),
            ],
        ),
        QuickNavSection::new(
            "derivatives",
            "Derivatives",
            vec![
                QuickNavItem::new("option_chain", "Option Chain", "[FO]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::FNO))),
                QuickNavItem::new("fno_oi", "F&O OI", "[FO2]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::FNO_OI))),
                QuickNavItem::new("fno_greeks", "Greeks", "[FO3]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::FNO_GREEKS))),
                QuickNavItem::new("pcr", "PCR", "[FO4]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::FNO_PCR))),
            ],
        ),
        QuickNavSection::new(
            "research",
            "Research",
            vec![
                QuickNavItem::new("earnings", "Earnings", "[R1]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::EARNINGS))),
                QuickNavItem::new("insider", "Insider", "[R2]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::INSIDER))),
                QuickNavItem::new("dividends", "Dividends", "[R3]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::DIVIDENDS))),
                QuickNavItem::new("intelligence", "Intelligence", "[R4]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::INTELLIGENCE))),
            ],
        ),
        QuickNavSection::new(
            "labs",
            "Labs",
            vec![
                QuickNavItem::new("backtest", "Backtest", "[F9]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::BACKTESTING))),
                QuickNavItem::new("strategy", "Strategy Studio", "[F2]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::STRATEGY))),
                QuickNavItem::new("model_lab", "Model Lab", "[L1]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::MODEL_LAB))),
                QuickNavItem::new("stat_lab", "Stat Lab", "[L2]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::STAT_LAB))),
            ],
        ),
        QuickNavSection::new(
            "portfolio",
            "Portfolio",
            vec![
                QuickNavItem::new("portfolio_hq", "Portfolio HQ", "[P1]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::PORTFOLIO))),
                QuickNavItem::new("tradebook", "Tradebook", "[P2]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::TRADEBOOK))),
                QuickNavItem::new("positions", "Positions", "[P3]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::PORTFOLIO))),
                QuickNavItem::new("funds", "Funds", "[P4]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::FUNDS))),
            ],
        ),
        QuickNavSection::new(
            "intel",
            "Intel",
            vec![
                QuickNavItem::new("intel_wire", "Intel Wire", "[I1]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::NEWS))),
                QuickNavItem::new("ideas", "Ideas Radar", "[I2]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::HOTLISTS))),
                QuickNavItem::new("research_dossiers", "Dossiers", "[I3]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::RESEARCH_DOSSIERS))),
                QuickNavItem::new("security_hub", "Security Hub", "[I4]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::SECURITY_HUB))),
            ],
        ),
        QuickNavSection::new(
            "workspace",
            "Workspace",
            vec![
                QuickNavItem::new("launchpad", "Launchpad", "[LP]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::LAUNCHPAD))),
                QuickNavItem::new("chart_workstation", "Workstation", "[WS]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::CHART_WORKSTATION))),
                QuickNavItem::new("workbench", "Workbench", "[WB]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::WORKBENCH))),
                QuickNavItem::new("simulator", "Simulator", "[SIM]")
                    .with_on_select(Callback::new(move |_| go_to(ROUTES::SIMULATOR))),
            ],
        ),
    ];

    view! {
        <div class="ot-home-page">
            {/* 3.1 Header Ribbon */}
            <header class="ot-home-widget-header-ribbon">
                <div class="ot-home-ribbon-body">
                    <div class="ot-home-ribbon-left">
                        <span class="ot-home-ribbon-kicker">"MISSION CONTROL"</span>
                        <h1 class="ot-home-ribbon-title">"Mission Control"</h1>
                        <div class="ot-home-ribbon-meta">
                            <span class="ot-home-ribbon-badge">{user_name}</span>
                            <span class="ot-home-ribbon-badge ot-home-ribbon-badge--pill">"IN"</span>
                            <span class="ot-home-ribbon-badge">"General"</span>
                            <span class="ot-home-ribbon-badge">"INR"</span>
                            <span class="ot-home-ribbon-badge">"Live"</span>
                        </div>
                        <div class="ot-home-ribbon-actions">
                            <HfButton
                                variant=HfButtonVariant::Ghost
                                size=HfButtonSize::Small
                                on_click=Callback::new(move |_| go_to(ROUTES::PORTFOLIO))
                                class="ot-home-ribbon-btn"
                            >
                                <span class="ot-home-ribbon-btn-label">"Portfolio HQ"</span>
                            </HfButton>
                            <HfButton
                                variant=HfButtonVariant::Ghost
                                size=HfButtonSize::Small
                                on_click=Callback::new(move |_| go_to(ROUTES::LAUNCHPAD))
                                class="ot-home-ribbon-btn"
                            >
                                <span class="ot-home-ribbon-btn-label">"Launchpad"</span>
                            </HfButton>
                            <HfButton
                                variant=HfButtonVariant::Ghost
                                size=HfButtonSize::Small
                                on_click=Callback::new(move |_| go_to(ROUTES::NEWS))
                                class="ot-home-ribbon-btn"
                            >
                                <span class="ot-home-ribbon-btn-label">"Intel Wire"</span>
                            </HfButton>
                            <HfButton
                                variant=HfButtonVariant::Ghost
                                size=HfButtonSize::Small
                                on_click=Callback::new(move |_| go_to(ROUTES::CHART_WORKSTATION))
                                class="ot-home-ribbon-btn"
                            >
                                <span class="ot-home-ribbon-btn-label">"Workstation"</span>
                            </HfButton>
                        </div>
                    </div>
                    <div class="ot-home-ribbon-right">
                        <LiveClockStrip />
                    </div>
                </div>
            </header>

            {/* 3.2 Market Heat Strip */}
            <div class="ot-home-widget-heat-strip-wrap">
                <MarketHeatStrip items=heat_items />
            </div>

            {/* 3.3 Grid Layout */}
            <div class="ot-home-widget-grid">
                <div class="ot-home-grid-left">
                    <MetricCard
                        label="Net Liquidation"
                        value="₹12,45,678"
                        delta=MetricCardDelta::new("+₹12,450 (+1.0 %)", MetricTone::Up)
                        tone=MetricTone::Accent
                        details=vec![
                            MetricCardDetail::new("Buying Power", "₹8,90,000"),
                            MetricCardDetail::new("Margin Used", "₹3,55,678")
                                .with_tone(MetricTone::Neutral),
                        ]
                    />
                    <PortfolioMiniChart points=portfolio_points />
                </div>
                <div class="ot-home-grid-right">
                    <SystemHealthBar items=health_items />
                    <IntelWire items=intel_items limit=5 />
                </div>
                <div class="ot-home-grid-bottom">
                    <QuickNavGrid sections=sections />
                </div>
            </div>
        </div>
    }
}
