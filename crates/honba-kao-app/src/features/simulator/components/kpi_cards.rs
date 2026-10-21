//! The four institutional KPI cards — the Leptos port of the
//! `.sim-kpi-grid` block on honba-fe `SimulatorView.tsx`'s overview tab.
//!
//! Like fe, three cards read the replay engine's live books (net simulated
//! P&L, profit factor & win rate over the engine's round trips, max
//! drawdown over the realised equity steps) and the fourth reads the
//! server run's reported Sharpe.
//!
//! Honesty deviations from fe:
//! - fe fell back to a hard-coded `2.14` Sharpe when no run reported one;
//!   here an absent metric reads `—` with the reason, never a number;
//! - the engine keeps its trips latest-first; the drawdown curve is built
//!   in chronological order (fe walked the latest-first list).

use honba_api_wasm::analytics::{MetricKind, format_metric_value, round_trip_stats};
use honba_api_wasm::pairing::{RoundTrip, RoundTripStatus};
use leptos::prelude::*;

use crate::features::simulator::components::replay_controls::money_signed_grouped;
use crate::features::simulator::kernel::engine::controller::SimulationEngineState;
use crate::features::simulator::kernel::ops::max_drawdown_pct;

/// The realised equity steps behind the max-drawdown card: the initial
/// capital, then one point per closed trip in chronological order (the
/// engine stores trips latest-first).
fn realised_equity_points(initial_capital: f64, trips_latest_first: &[RoundTrip]) -> Vec<f64> {
    let mut running = initial_capital;
    let mut points = vec![running];
    for trip in trips_latest_first.iter().rev() {
        if trip.status == RoundTripStatus::Closed
            && let Some(pnl) = trip.realized_pnl
        {
            running += pnl;
            points.push(running);
        }
    }
    points
}

/// `color-bullish` / `color-bearish` for a signed value (fe's ternary).
fn tone(value: f64) -> &'static str {
    if value >= 0.0 { "color-bullish" } else { "color-bearish" }
}

#[component]
pub fn KpiCards(
    /// The page's engine-state memo.
    #[prop(into)]
    engine_state: Signal<SimulationEngineState>,
    /// The server run's reported Sharpe, when a run reported metrics.
    #[prop(into)]
    sharpe: Signal<Option<f64>>,
) -> impl IntoView {
    let net_pnl =
        Memo::new(move |_| engine_state.with(|state| state.realized_pnl + state.unrealized_pnl));
    let total_return = Memo::new(move |_| {
        engine_state.with(|state| {
            if state.initial_capital > 0.0 {
                (state.realized_pnl + state.unrealized_pnl) / state.initial_capital
            } else {
                0.0
            }
        })
    });
    let stats = Memo::new(move |_| engine_state.with(|state| round_trip_stats(&state.round_trips)));
    let max_drawdown = Memo::new(move |_| {
        engine_state.with(|state| {
            max_drawdown_pct(&realised_equity_points(state.initial_capital, &state.round_trips))
        })
    });

    view! {
        <div class="sim-kpi-grid">
            <div class="sim-kpi-card">
                <span class="sim-kpi-lbl">"Net Simulated P&L"</span>
                <div class=move || format!("sim-kpi-val {}", tone(total_return.get()))>
                    {move || money_signed_grouped(net_pnl.get())}
                </div>
                <span class=move || format!("sim-kpi-sub {}", tone(total_return.get()))>
                    {move || {
                        let pct = total_return.get() * 100.0;
                        format!("{}{pct:.2}% overall return", if pct >= 0.0 { "+" } else { "" })
                    }}
                </span>
            </div>

            <div class="sim-kpi-card">
                <span class="sim-kpi-lbl">"Profit Factor & Win Rate"</span>
                <div class="sim-kpi-val">
                    {move || format_metric_value(stats.get().profit_factor, MetricKind::Ratio)}
                    <span class="sim-kpi-val-aside">
                        {move || {
                            let rate = stats
                                .get()
                                .win_rate
                                .map_or_else(|| "—".to_string(), |rate| format!("{:.1}%", rate * 100.0));
                            format!("({rate})")
                        }}
                    </span>
                </div>
                <span class="sim-kpi-sub">
                    {move || {
                        let stats = stats.get();
                        format!("{} wins / {} losses ({} closed)", stats.wins, stats.losses, stats.closed)
                    }}
                </span>
            </div>

            <div class="sim-kpi-card">
                <span class="sim-kpi-lbl">"Max Drawdown"</span>
                <div class="sim-kpi-val color-bearish">
                    {move || {
                        let drawdown = max_drawdown.get();
                        if drawdown > 0.0 { format!("-{drawdown:.2}%") } else { "0.00%".to_string() }
                    }}
                </div>
                <span class="sim-kpi-sub">
                    {move || {
                        let equity = engine_state.with(|state| state.equity);
                        format!("Equity: ₹{}", format_metric_value(Some(equity.round()), MetricKind::Currency)
                            .trim_end_matches(".00"))
                    }}
                </span>
            </div>

            <div class="sim-kpi-card">
                <span class="sim-kpi-lbl">"Sharpe / Risk-Adjusted"</span>
                <div class="sim-kpi-val">
                    {move || format_metric_value(sharpe.get(), MetricKind::Ratio)}
                </div>
                <span class="sim-kpi-sub">
                    {move || {
                        if sharpe.get().is_some() {
                            "Annualized, reported by the server run"
                        } else {
                            "Reported once a server backtest completes"
                        }
                    }}
                </span>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honba_api_wasm::pairing::{Direction, RoundTripExitReason};

    fn trip(id: &str, pnl: Option<f64>, status: RoundTripStatus) -> RoundTrip {
        RoundTrip {
            id: id.to_string(),
            instrument: "TCS.NSE".to_string(),
            direction: Direction::Long,
            status,
            entry_ts: 0,
            exit_ts: None,
            quantity: 1.0,
            open_quantity: 0.0,
            avg_entry: 100.0,
            avg_exit: None,
            realized_pnl: pnl,
            return_pct: None,
            fees: 0.0,
            exit_reason: RoundTripExitReason::PositionClosed,
            entry_fills: Vec::new(),
            exit_fills: Vec::new(),
        }
    }

    #[test]
    fn equity_points_walk_closed_trips_oldest_first() {
        // Latest-first, as the engine stores them.
        let trips = [
            trip("t3", Some(-50.0), RoundTripStatus::Closed),
            trip("t2", None, RoundTripStatus::Open),
            trip("t1", Some(200.0), RoundTripStatus::Closed),
        ];
        assert_eq!(realised_equity_points(1_000.0, &trips), vec![1_000.0, 1_200.0, 1_150.0]);
        assert_eq!(realised_equity_points(1_000.0, &[]), vec![1_000.0]);
    }

    #[test]
    fn tone_treats_flat_as_bullish_like_fe() {
        assert_eq!(tone(0.0), "color-bullish");
        assert_eq!(tone(-0.01), "color-bearish");
    }
}
