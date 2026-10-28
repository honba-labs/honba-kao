use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

use crate::features::derivatives::*;
use crate::features::portfolio::*;
use crate::features::quant::*;
use crate::features::research::*;
use crate::features::risk_ops::*;
use crate::features::screener::*;
use crate::features::system::*;
use crate::features::trading::*;
use crate::features::wealth::*;
use crate::router::chrome::AppChrome;

#[component]
pub fn AppRouter() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <div class="p-6 text-sm text-text-muted">"404 - Page Not Found"</div> }>
                <ParentRoute path=path!("") view=AppChrome>
                    // System Pages
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("landing") view=LandingPage />
                    <Route path=path!("auth") view=AuthPage />
                    <Route path=path!("account") view=AccountPage />

                    // Cluster 1: Trading Cockpit
                    <Route path=path!("trading") view=CockpitPage />
                    <Route path=path!("trading/cockpit") view=CockpitPage />
                    <Route path=path!("trading/multi-chart") view=MultiChartPage />
                    <Route path=path!("trading/dom") view=DomPage />
                    <Route path=path!("trading/tape") view=TapePage />
                    <Route path=path!("trading/mta") view=MtaPage />
                    <Route path=path!("trading/compare") view=ComparePage />
                    <Route path=path!("trading/launchpad") view=LaunchpadPage />

                    // Cluster 2: Derivatives (F&O)
                    <Route path=path!("derivatives") view=OptionChainPage />
                    <Route path=path!("derivatives/options") view=OptionChainPage />
                    <Route path=path!("derivatives/option-chain") view=OptionChainPage />
                    <Route path=path!("derivatives/strategy") view=StrategyBuilderPage />
                    <Route path=path!("derivatives/greeks") view=GreeksHeatmapPage />
                    <Route path=path!("derivatives/oi") view=OiAnalysisPage />

                    // Cluster 3: Portfolio & Margin Hub
                    <Route path=path!("portfolio") view=PortfolioPage />
                    <Route path=path!("portfolio/tradebook") view=TradebookPage />
                    <Route path=path!("portfolio/shadow") view=ShadowAccountPage />
                    <Route path=path!("portfolio/optimizer") view=LabOptimizerPage />

                    // Cluster 4: Mutual Funds & Wealth
                    <Route path=path!("wealth") view=MutualFundsPage />
                    <Route path=path!("wealth/mutual-funds") view=MutualFundsPage />
                    <Route path=path!("wealth/sip") view=SipPlannerPage />
                    <Route path=path!("wealth/allocation") view=AssetAllocationPage />
                    <Route path=path!("wealth/bonds") view=BondsPage />

                    // Cluster 5: Quantitative Research & Simulator
                    <Route path=path!("quant") view=SimulatorPage />
                    <Route path=path!("quant/simulator") view=SimulatorPage />
                    <Route path=path!("quant/backtesting") view=BacktestingDeckPage />
                    <Route path=path!("quant/model-lab") view=ModelLabPage />
                    <Route path=path!("quant/stat-lab") view=StatLabPage />
                    <Route path=path!("quant/pair-trading") view=PairTradingPage />
                    <Route path=path!("quant/strategy-studio") view=StrategyStudioPage />

                    // Cluster 6: Stock Screener
                    <Route path=path!("screener") view=ScreenerPage />

                    // Cluster 7: Research & Dossiers
                    <Route path=path!("research") view=SecurityHubPage />
                    <Route path=path!("research/security") view=SecurityHubPage />
                    <Route path=path!("research/dossiers") view=DossiersPage />
                    <Route path=path!("research/timeline") view=TimelinePage />
                    <Route path=path!("research/markets") view=MarketsPage />

                    // Cluster 8: Risk & Operations
                    <Route path=path!("risk") view=RiskPage />
                    <Route path=path!("risk/oms") view=OmsPage />
                    <Route path=path!("risk/ops") view=OpsPage />
                    <Route path=path!("risk/sizer") view=PositionSizerPage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
