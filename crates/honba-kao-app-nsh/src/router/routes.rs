//! Path → page element mapping — NSH port per nsh-port.md.
//!
//! Replaces `leptos_router`'s `<Routes><Route .../></Routes>` tree. Called
//! from [`crate::router::chrome::app_chrome`] via `nsh_core::prelude::dynamic(..)`,
//! which is the path-reactive substitute for `leptos_router`'s `<Outlet/>`.
//!
//! Every route from the original `leptos_router` tree is preserved exactly,
//! including the "Legacy v2 route aliases" section (see
//! `crate::router::paths`'s `ROUTE_LEGACY_*` constants), with a single 404
//! fallback matching the original `<Routes fallback=...>`.

use crate::features::derivatives::*;
use crate::features::portfolio::*;
use crate::features::quant::*;
use crate::features::research::*;
use crate::features::risk_ops::*;
use crate::features::screener::*;
use crate::features::system::*;
use crate::features::trading::*;
use crate::features::wealth::*;
use honba_kao_ui_core::theme::tokens::HonbaStyle;
use nsh_core::prelude::*;

/// Resolves `path` to the element of the page that owns it, falling back to
/// a 404 notice for anything unmatched (mirrors the original `<Routes
/// fallback=|| view! { <div>"404 - Page Not Found"</div> }>`).
pub fn route_element(path: &str) -> AnyElement {
    match path {
        // System Pages
        "" | "/" => HomePage::element(Default::default()).into_element(),
        "/landing" => LandingPage::element(Default::default()).into_element(),
        "/auth" => AuthPage::element(Default::default()).into_element(),
        "/account" => AccountPage::element(Default::default()).into_element(),

        // Cluster 1: Trading Cockpit
        "/trading" | "/trading/cockpit" => CockpitPage::element(Default::default()).into_element(),
        "/trading/multi-chart" => MultiChartPage::element(Default::default()).into_element(),
        "/trading/dom" => DomPage::element(Default::default()).into_element(),
        "/trading/tape" => TapePage::element(Default::default()).into_element(),
        "/trading/mta" => MtaPage::element(Default::default()).into_element(),
        "/trading/compare" => ComparePage::element(Default::default()).into_element(),
        "/trading/launchpad" => LaunchpadPage::element(Default::default()).into_element(),

        // Cluster 2: Derivatives (F&O)
        "/derivatives" | "/derivatives/options" | "/derivatives/option-chain" => {
            OptionChainPage::element(Default::default()).into_element()
        }
        "/derivatives/strategy" => StrategyBuilderPage::element(Default::default()).into_element(),
        "/derivatives/greeks" => GreeksHeatmapPage::element(Default::default()).into_element(),
        "/derivatives/oi" => OiAnalysisPage::element(Default::default()).into_element(),

        // Cluster 3: Portfolio & Margin Hub
        "/portfolio" => PortfolioPage::element(Default::default()).into_element(),
        "/portfolio/tradebook" => TradebookPage::element(Default::default()).into_element(),
        "/portfolio/shadow" => ShadowAccountPage::element(Default::default()).into_element(),
        "/portfolio/optimizer" => LabOptimizerPage::element(Default::default()).into_element(),

        // Cluster 4: Mutual Funds & Wealth
        "/wealth" | "/wealth/mutual-funds" => {
            MutualFundsPage::element(Default::default()).into_element()
        }
        "/wealth/sip" => SipPlannerPage::element(Default::default()).into_element(),
        "/wealth/allocation" => AssetAllocationPage::element(Default::default()).into_element(),
        "/wealth/bonds" => BondsPage::element(Default::default()).into_element(),

        // Cluster 5: Quantitative Research & Simulator
        "/quant" | "/quant/simulator" => SimulatorPage::element(Default::default()).into_element(),
        "/quant/backtesting" => BacktestingDeckPage::element(Default::default()).into_element(),
        "/quant/model-lab" => ModelLabPage::element(Default::default()).into_element(),
        "/quant/stat-lab" => StatLabPage::element(Default::default()).into_element(),
        "/quant/pair-trading" => PairTradingPage::element(Default::default()).into_element(),
        "/quant/strategy-studio" => StrategyStudioPage::element(Default::default()).into_element(),

        // Cluster 6: Stock Screener
        "/screener" => ScreenerPage::element(Default::default()).into_element(),

        // Cluster 7: Research & Dossiers
        "/research" | "/research/security" => {
            SecurityHubPage::element(Default::default()).into_element()
        }
        "/research/dossiers" => DossiersPage::element(Default::default()).into_element(),
        "/research/timeline" => TimelinePage::element(Default::default()).into_element(),
        "/research/markets" => MarketsPage::element(Default::default()).into_element(),

        // Cluster 8: Risk & Operations
        "/risk" => RiskPage::element(Default::default()).into_element(),
        "/risk/oms" => OmsPage::element(Default::default()).into_element(),
        "/risk/ops" => OpsPage::element(Default::default()).into_element(),
        "/risk/sizer" => PositionSizerPage::element(Default::default()).into_element(),

        // ---- Legacy v2 route aliases (preserved bookmarks / deep links) ----
        // See router/paths.rs ROUTE_LEGACY_* constants.
        "/workbench" => CockpitPage::element(Default::default()).into_element(),
        "/fno" => OptionChainPage::element(Default::default()).into_element(),
        "/fno/strategy" => StrategyBuilderPage::element(Default::default()).into_element(),
        "/fno/oi" => OiAnalysisPage::element(Default::default()).into_element(),
        "/fno/greeks" => GreeksHeatmapPage::element(Default::default()).into_element(),
        "/equity/stocks" => CockpitPage::element(Default::default()).into_element(),
        "/equity/dom" => DomPage::element(Default::default()).into_element(),
        "/equity/tape" => TapePage::element(Default::default()).into_element(),
        "/equity/chart-workstation" => MultiChartPage::element(Default::default()).into_element(),
        "/equity/mta" => MtaPage::element(Default::default()).into_element(),
        "/equity/compare" => ComparePage::element(Default::default()).into_element(),
        "/backtesting" => BacktestingDeckPage::element(Default::default()).into_element(),
        "/backtesting/model-lab" => ModelLabPage::element(Default::default()).into_element(),
        "/settings" => AccountPage::element(Default::default()).into_element(),

        _ => view()
            .px_4()
            .py_3()
            .child(
                text("404 - Page Not Found")
                    .text_size(px(13.0))
                    .text_muted(),
            )
            .into_element(),
    }
}
