use honba_kao_ui_core::molecules::quant::HfBacktestMetricCard;
use honba_kao_ui_core::templates::HfPage;
use leptos::prelude::*;

#[component]
pub fn BacktestingDeckPage() -> impl IntoView {
    view! {
        <HfPage title="Quantitative Strategy Backtesting" subtitle="Walk-forward optimization, parameter sweeps, and execution slippage breakdown">
            <div class="grid grid-cols-1 laptop:grid-cols-4 gap-4">
                <HfBacktestMetricCard
                    label="CAGR"
                    value="24.5%"
                    positive=true
                />
                <HfBacktestMetricCard
                    label="Sharpe Ratio"
                    value="1.85"
                    positive=true
                />
                <HfBacktestMetricCard
                    label="Max Drawdown"
                    value="-11.2%"
                    positive=false
                />
                <HfBacktestMetricCard
                    label="Win Rate"
                    value="58.5%"
                    positive=true
                />
            </div>
        </HfPage>
    }
}
