//! The route table.
//!
//! Signed-in users land on Portfolio (the console's default workspace);
//! unknown paths fall back to `/`, which itself redirects authenticated users
//! there and still shows the marketing landing to guests.
//!
//! The gated workspace screens are children of one pathless parent route
//! whose view, [`AppShell`], mounts the [`AppChrome`] once and renders the
//! matched page through `<Outlet/>`. Navigating between workspace screens
//! therefore swaps only the outlet content; the header, nav rail and their
//! state survive.

use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Redirect, Route, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;

use crate::features::account::{FundsPage, SettingsPage};
use crate::features::auth::{LoginPage, RegisterPage, ResetPasswordPage, SetupPage};
use crate::features::backtesting::backtesting_page::BacktestingDeckPage;
use crate::features::backtesting::BacktestingPage;
use crate::features::chart_workstation::ChartWorkstationPage;
use crate::features::compare::ComparePage;
use crate::features::dom::DomPage;
use crate::features::dossiers::DossiersPage;
use crate::features::fno::{
    ExpiryPage, FlowPage, FnoHeatmapPage, FuturesPage, GreeksHeatmapPage, OiPage,
    OptionChainPage, OptionGreeksPage, PcrPage, StrategyBuilderPage,
};
use crate::features::landing::LandingPage;
use crate::features::launchpad::{LaunchpadPage, PopoutPage};
use crate::features::markets::{
    BondsPage, CommoditiesPage, CryptoPage, EconomicsPage, EtfPage, ForexPage, HeatmapPage,
    HotlistsPage, MarketPage, MonitoringPage, MutualFundsPage, NewsPage, WatchlistPage,
    YieldCurvePage,
};
use crate::features::model_lab::ModelLabPage;
use crate::features::mta::MtaPage;
use crate::features::portfolio::PortfolioPage;
use crate::features::portfolio_extras::{AlertsPage, JournalPage, PaperPage};
use crate::features::portfolio_lab::PortfolioLabPage;
use crate::features::quant_extras::{
    AlgoFrameworkPage, AlphaZooPage, ResearchAutopilotPage, StrategyExportPage,
};
use crate::features::research::{
    DividendsPage, EarningsPage, FactorsPage, InsiderPage, IntelligenceTimelinePage,
    RelativeStrengthPage, ResearchLibraryPage, SectorRotationPage,
};
use crate::features::stat_lab::StatLabPage;
use crate::features::stat_lab::stat_lab_page::StatLabPageContent;
use crate::features::pair_trading::PairTradingPage;
use crate::features::pair_trading::pair_trading_page::PairTradingPageContent;
use crate::features::screener::ScreenerPage;
use crate::features::security_hub::SecurityHubPage;
use crate::features::risk::RiskPage;
use crate::features::risk_ops_extras::{CockpitPage, CorrelationPage, DataQualityPage, ReportsPage};
use crate::features::oms::OmsPage;
use crate::features::ops::OpsPage;
use crate::features::position_sizer::PositionSizerPage;
use crate::features::shadow_account::ShadowAccountPage;
use crate::features::simulator::SimulatorPage;
use crate::features::strategy_studio::StrategyStudioPage;
use crate::features::system_extras::{AboutPage, AccountHomePage, PluginsPage, SavedViewsPage};
use crate::features::tape::TapePage;
use crate::features::tradebook::TradebookPage;
use crate::features::workbench::WorkbenchPage;
use crate::router::app_chrome::AppChrome;
use crate::router::paths::{get_current_page_for_path, Page, ROUTES};
use crate::shared::contexts::use_auth;

#[component]
pub fn AppRoutes() -> impl IntoView {
    view! {
        // Unknown paths fall through to `/`, which routes by session state.
        <Routes fallback=|| view! { <Redirect path=ROUTES::HOME /> }>
            // --- Public, full-bleed screens -----------------------------------
            <Route path=path!("/") view=|| view! { <RootRoute /> } />
            <Route path=path!("/login") view=|| view! { <LoginPage /> } />
            <Route path=path!("/register") view=|| view! { <RegisterPage /> } />
            <Route path=path!("/reset-password") view=|| view! { <ResetPasswordPage /> } />
            <Route path=path!("/setup") view=|| view! { <SetupPage /> } />
            // Pop-out window: a bare single panel, no shell chrome.
            <Route path=path!("/equity/launchpad/popout") view=|| view! { <PopoutPage /> } />

            // --- Gated workspace (persistent chrome) --------------------------
            <ParentRoute path=path!("") view=AppShell>
                // --- Markets & Overview --------------------------------------
                <Route path=path!("/equity/stocks") view=MarketPage />
                <Route path=path!("/equity/monitoring") view=MonitoringPage />
                <Route path=path!("/equity/watchlist") view=WatchlistPage />
                <Route path=path!("/equity/hotlists") view=HotlistsPage />
                <Route path=path!("/equity/heatmap") view=HeatmapPage />
                <Route path=path!("/equity/news") view=NewsPage />
                <Route path=path!("/equity/economics") view=EconomicsPage />
                <Route path=path!("/equity/commodities") view=CommoditiesPage />
                <Route path=path!("/equity/forex") view=ForexPage />
                <Route path=path!("/equity/crypto") view=CryptoPage />
                <Route path=path!("/equity/bonds") view=BondsPage />
                <Route path=path!("/equity/yield-curve") view=YieldCurvePage />
                <Route path=path!("/equity/etf-analytics") view=EtfPage />
                <Route path=path!("/equity/mutual-funds") view=MutualFundsPage />

                <Route path=path!("/portfolio") view=PortfolioPage />
                <Route path=path!("/screener") view=ScreenerPage />
                <Route path=path!("/equity/security/:ticker") view=SecurityHubPage />

                // --- Research --------------------------------------------------
                <Route path=path!("/equity/factors") view=FactorsPage />
                <Route path=path!("/equity/insider") view=InsiderPage />
                <Route path=path!("/equity/dividends") view=DividendsPage />
                <Route path=path!("/equity/earnings") view=EarningsPage />
                <Route path=path!("/equity/intelligence-timeline") view=IntelligenceTimelinePage />
                <Route path=path!("/equity/research-dossiers") view=DossiersPage />
                <Route path=path!("/equity/research") view=ResearchLibraryPage />
                <Route path=path!("/equity/sector-rotation") view=SectorRotationPage />
                <Route path=path!("/equity/rs") view=RelativeStrengthPage />

                <Route path=path!("/fno") view=OptionChainPage />
                <Route path=path!("/fno/strategy") view=StrategyBuilderPage />
                <Route path=path!("/fno/oi") view=OiPage />
                <Route path=path!("/fno/greeks") view=GreeksHeatmapPage />
                <Route path=path!("/fno/futures") view=FuturesPage />
                <Route path=path!("/fno/flow") view=FlowPage />
                <Route path=path!("/fno/pcr") view=PcrPage />
                <Route path=path!("/fno/heatmap") view=FnoHeatmapPage />
                <Route path=path!("/fno/expiry") view=ExpiryPage />
                <Route path=path!("/equity/option-greeks") view=OptionGreeksPage />
                <Route path=path!("/equity/chart-workstation") view=ChartWorkstationPage />
                <Route path=path!("/equity/compare") view=ComparePage />
                <Route path=path!("/equity/mta") view=MtaPage />
                <Route path=path!("/equity/dom") view=DomPage />
                <Route path=path!("/equity/tape") view=TapePage />
                <Route path=path!("/equity/launchpad") view=LaunchpadPage />
                <Route path=path!("/workbench") view=WorkbenchPage />
                <Route path=path!("/strategy") view=StrategyStudioPage />
                <Route path=path!("/simulator") view=SimulatorPage />
                // Phase 5: Quant Suite
                <Route path=path!("/backtesting") view=BacktestingPage />
                <Route path=path!("/backtesting/deck") view=BacktestingDeckPage />
                <Route path=path!("/backtesting/model-lab") view=ModelLabPage />
                <Route path=path!("/backtesting/model-governance") view=ModelLabPage />
                <Route path=path!("/backtesting/algorithm-framework") view=AlgoFrameworkPage />
                <Route path=path!("/equity/alpha-zoo") view=AlphaZooPage />
                <Route path=path!("/equity/research-autopilot") view=ResearchAutopilotPage />
                <Route path=path!("/equity/strategy-export") view=StrategyExportPage />
                <Route path=path!("/equity/portfolio/lab") view=PortfolioLabPage />
                <Route path=path!("/equity/portfolio/optimizer") view=PortfolioLabPage />
                    // Phase 5.6: Statistical & Pair Trading Labs
                    <Route path=path!("/equity/stat-lab") view=StatLabPage />
                    <Route path=path!("/equity/stat-lab/details") view=StatLabPageContent />
                    <Route path=path!("/equity/pair-trading") view=PairTradingPage />
                    <Route path=path!("/equity/pair-trading/details") view=PairTradingPageContent />
                <Route path=path!("/tradebook") view=TradebookPage />

                // --- Portfolio & Execution --------------------------------------
                <Route path=path!("/equity/paper") view=PaperPage />
                <Route path=path!("/equity/journal") view=JournalPage />
                <Route path=path!("/equity/alerts") view=AlertsPage />

                // Phase 6: Risk Engine, OMS Compliance, Ops Cockpit & Position Sizing
                <Route path=path!("/equity/risk") view=RiskPage />
                <Route path=path!("/equity/oms") view=OmsPage />
                <Route path=path!("/equity/ops") view=OpsPage />
                <Route path=path!("/equity/position-sizer") view=PositionSizerPage />
                <Route path=path!("/equity/shadow-account") view=ShadowAccountPage />
                <Route path=path!("/equity/correlation") view=CorrelationPage />
                <Route path=path!("/equity/cockpit") view=CockpitPage />
                <Route path=path!("/equity/data-quality") view=DataQualityPage />
                <Route path=path!("/equity/reports") view=ReportsPage />

                // --- System ------------------------------------------------------
                <Route path=path!("/equity/plugins") view=PluginsPage />
                <Route path=path!("/equity/saved-views") view=SavedViewsPage />
                <Route path=path!("/account") view=AccountHomePage />
                <Route path=path!("/about") view=AboutPage />

                // --- User cluster -------------------------------------------
                <Route path=path!("/settings") view=SettingsPage />
                <Route path=path!("/funds") view=FundsPage />
            </ParentRoute>
        </Routes>
    }
}

/// Layout for the gated workspace: guests are sent to sign-in, hosts get the
/// chrome mounted once with the matched child page in its `<main>` outlet.
///
/// The session is checked when the shell mounts. The shell is only left
/// mounted across navigations between its own children, and the only way a
/// session ends (sign-out) navigates to `/`, which unmounts the shell.
#[component]
fn AppShell() -> impl IntoView {
    let auth = use_auth();

    if !auth.is_authenticated() {
        return view! { <Redirect path=ROUTES::LOGIN /> }.into_any();
    }

    let location = use_location();
    let current_page = Memo::new(move |_| {
        location
            .pathname
            .with(|pathname| get_current_page_for_path(pathname).unwrap_or(Page::Home))
    });

    view! {
        <AppChrome current_page=current_page>
            <Outlet />
        </AppChrome>
    }
    .into_any()
}

/// `/`: authenticated users go straight to Portfolio (the default landing);
/// guests see the marketing landing.
#[component]
fn RootRoute() -> impl IntoView {
    let auth = use_auth();

    view! {
        {if auth.is_authenticated() {
            view! { <Redirect path=ROUTES::PORTFOLIO /> }.into_any()
        } else {
            view! { <LandingPage /> }.into_any()
        }}
    }
}
