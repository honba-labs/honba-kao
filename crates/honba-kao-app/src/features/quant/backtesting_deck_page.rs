use honba_kao_ui_core::molecules::quant::HfBacktestMetricCard;
use honba_kao_ui_core::templates::HfMosaicWorkspace;
use leptos::prelude::*;

#[component]
pub fn BacktestingDeckPage() -> impl IntoView {
    view! {
        <HfMosaicWorkspace>
            <div class="col-span-12 laptop:col-span-12 p-4 grid grid-cols-1 laptop:grid-cols-4 gap-4 h-full">
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
            <div class="col-span-12 laptop:col-span-12 p-4 rounded border border-border-soft bg-surface-base h-full">
                <span class="text-xs font-bold text-text-high font-mono block mb-1">"Equity Curve & Drawdown"</span>
                <div class="h-[300px] flex items-center justify-center text-xs text-text-muted border border-border-soft rounded bg-surface-subtle">
                    "Walk-forward optimization and parameter sweeps rendered via api-wasm analytics."
                </div>
            </div>
        </HfMosaicWorkspace>
    }
}
