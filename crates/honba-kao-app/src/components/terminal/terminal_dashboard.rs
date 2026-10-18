//! Terminal dashboard — the console screen. Mirrors
//! `web/src/components/terminal/TerminalDashboard.tsx`: KPIs over a grid of
//! chart / order book / positions / orders / decision log panels, all polled.

use leptos::prelude::*;

use crate::components::common::StatusBar;
use crate::components::terminal::{
    ExecutionLog, KlineChart, OrderBook, OrdersTable, PositionsTable, RiskRadar, SignalMatrix,
};
use crate::components::trader::DecisionLog;
use crate::components::ui::Kpi;
use crate::hooks::use_interval;
use crate::types::{AccountInfo, SystemStatus, TraderInfo};
use crate::utils::{compact_currency, currency, percent};

#[component]
pub fn TerminalDashboard(
    selected_trader: Option<TraderInfo>,
    status: Option<SystemStatus>,
    account: Option<AccountInfo>,
) -> impl IntoView {
    // SWR-equivalent: a 15s poll tick read by the header line.
    let tick = use_interval(15_000);

    let account = account.unwrap_or_else(crate::data::demo_data::demo_account);
    let trader_name = selected_trader
        .as_ref()
        .map(|t| t.trader_name.clone())
        .unwrap_or_else(|| "All traders".to_string());
    let running = status.as_ref().map(|s| s.running).unwrap_or(true);
    let latency = status.as_ref().map(|s| s.latency_ms).unwrap_or(14);
    let day_pnl_pct = if account.total_value > 0.0 {
        (account.day_pnl / account.total_value) * 100.0
    } else {
        0.0
    };
    let pnl_up = account.day_pnl >= 0.0;

    view! {
        <div class="console">
            <div class="page-header">
                <div>
                    <h1 class="page-header__title">"Trader dashboard"</h1>
                    <p class="page-header__sub">
                        {move || {
                            format!(
                                "{} · {} · auto-refresh 15s · tick {}",
                                trader_name,
                                if running { "running" } else { "paused" },
                                tick.get(),
                            )
                        }}
                    </p>
                </div>
                <div class="page-header__actions">
                    <span class=move || format!("mode-pill {}", if running { "sandbox" } else { "live" })>
                        {move || if running { "● ENGINE LIVE" } else { "○ ENGINE PAUSED" }}
                    </span>
                </div>
            </div>

            <div class="kpi-grid">
                <Kpi label="Total value" delta="+1.8%" up=true>
                    {move || compact_currency(account.total_value)}
                </Kpi>
                <Kpi label="Day P&L" delta=percent(day_pnl_pct) up=pnl_up>
                    {move || currency(account.day_pnl)}
                </Kpi>
                <Kpi label="Buying power">{move || compact_currency(account.buying_power)}</Kpi>
                <Kpi label="Engine latency">{move || format!("{} ms", latency)}</Kpi>
            </div>

            <div class="grid-2">
                <KlineChart />
                <OrderBook />
            </div>

            <PositionsTable />
            <OrdersTable />

            <div class="grid-2">
                <DecisionLog />
                <ExecutionLog />
            </div>

            <div class="grid-2">
                <RiskRadar />
                <SignalMatrix />
            </div>

            <StatusBar />
        </div>
    }
}
