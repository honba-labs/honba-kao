//! Page-mount tests.
//!
//! Verifies that every page component across the 8 domain clusters is present
//! and referenceable (compile-time). These tests ensure the 8-cluster feature
//! surface is intact; rendering tests require a WASM/DOM harness and are out
//! of scope for the host test runner.

/// Derivatives cluster
#[allow(unused_imports)]
use honba_kao_app::features::derivatives::{
    GreeksHeatmapPage, OiAnalysisPage, OptionChainPage, StrategyBuilderPage,
};
/// Portfolio cluster
#[allow(unused_imports)]
use honba_kao_app::features::portfolio::{
    LabOptimizerPage, PortfolioPage, ShadowAccountPage, TradebookPage,
};
/// Quant cluster
#[allow(unused_imports)]
use honba_kao_app::features::quant::{
    BacktestingDeckPage, ModelLabPage, PairTradingPage, SimulatorPage, StatLabPage,
    StrategyStudioPage,
};
/// Research cluster
#[allow(unused_imports)]
use honba_kao_app::features::research::{DossiersPage, MarketsPage, SecurityHubPage, TimelinePage};
/// Risk & Ops cluster
#[allow(unused_imports)]
use honba_kao_app::features::risk_ops::{OmsPage, OpsPage, PositionSizerPage, RiskPage};
/// Screener cluster
#[allow(unused_imports)]
use honba_kao_app::features::screener::ScreenerPage;
/// System cluster
#[allow(unused_imports)]
use honba_kao_app::features::system::{AccountPage, AuthPage, HomePage, LandingPage};
/// Trading cluster
#[allow(unused_imports)]
use honba_kao_app::features::trading::{
    CockpitPage, ComparePage, DomPage, LaunchpadPage, MtaPage, MultiChartPage, TapePage,
};
/// Wealth cluster
#[allow(unused_imports)]
use honba_kao_app::features::wealth::{
    AssetAllocationPage, BondsPage, MutualFundsPage, SipPlannerPage,
};

#[test]
fn trading_cluster_pages_exist() {
    let _ = CockpitPage;
    let _ = MultiChartPage;
    let _ = DomPage;
    let _ = TapePage;
    let _ = MtaPage;
    let _ = ComparePage;
    let _ = LaunchpadPage;
}

#[test]
fn derivatives_cluster_pages_exist() {
    let _ = OptionChainPage;
    let _ = StrategyBuilderPage;
    let _ = GreeksHeatmapPage;
    let _ = OiAnalysisPage;
}

#[test]
fn portfolio_cluster_pages_exist() {
    let _ = PortfolioPage;
    let _ = TradebookPage;
    let _ = ShadowAccountPage;
    let _ = LabOptimizerPage;
}

#[test]
fn wealth_cluster_pages_exist() {
    let _ = MutualFundsPage;
    let _ = SipPlannerPage;
    let _ = AssetAllocationPage;
    let _ = BondsPage;
}

#[test]
fn quant_cluster_pages_exist() {
    let _ = SimulatorPage;
    let _ = BacktestingDeckPage;
    let _ = ModelLabPage;
    let _ = StatLabPage;
    let _ = PairTradingPage;
    let _ = StrategyStudioPage;
}

#[test]
fn screener_cluster_page_exists() {
    let _ = ScreenerPage;
}

#[test]
fn research_cluster_pages_exist() {
    let _ = SecurityHubPage;
    let _ = DossiersPage;
    let _ = TimelinePage;
    let _ = MarketsPage;
}

#[test]
fn risk_ops_cluster_pages_exist() {
    let _ = RiskPage;
    let _ = OmsPage;
    let _ = OpsPage;
    let _ = PositionSizerPage;
}

#[test]
fn system_cluster_pages_exist() {
    let _ = HomePage;
    let _ = LandingPage;
    let _ = AuthPage;
    let _ = AccountPage;
}
