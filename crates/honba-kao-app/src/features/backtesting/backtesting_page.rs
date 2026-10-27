use leptos::prelude::*;

/// Primary backtesting deck page — strategy templates, runner, and analytics.
#[component]
pub fn BacktestingDeckPage() -> impl IntoView {
    view! {
        <section class="prose max-w-3xl">
            <h2 class="text-xl font-semibold mb-4">Strategy Templates</h2>
            <ul class="space-y-3">
                <li class="flex items-center space-x-2">
                    <span class="w-3 h-3 rounded-full bg-primary"></span>
                    "Trend Following"
                </li>
                <li class="flex items-center space-x-2">
                    <span class="w-3 h-3 rounded-full bg-secondary"></span>
                    "Mean Reversion"
                </li>
                <li class="flex items-center space-x-2">
                    <span class="w-3 h-3 rounded-full bg-accent"></span>
                    "Statistical Arbitrage"
                </li>
                <li class="flex items-center space-x-2">
                    <span class="w-3 h-3 rounded-full bg-info"></span>
                    "Options Theta Decay"
                </li>
            </ul>
        </section>

        <section class="mt-8 grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
                <h3 class="font-medium mb-2">Backtest Runner</h3>
                <p class="text-sm text-muted-foreground">
                    "Configure strategy parameters, execution model, and run parameters."
                </p>
            </div>
            <div>
                <h3 class="font-medium mb-2">Analytics</h3>
                <p class="text-sm text-muted-foreground">
                    "Equity curve, drawdown, monthly PnL, and risk metrics."
                </p>
            </div>
        </section>
    }
}