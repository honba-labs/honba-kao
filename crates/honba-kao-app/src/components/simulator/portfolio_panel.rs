//! The portfolio dock panel — fe's `portfolio` surface, visible through
//! the replay engine's own books (P7).
//!
//! fe filled this from a sandbox account adapter; per the pinned port
//! decision, the engine's own ledger **is** the portfolio surface here:
//! realized/unrealized PnL and equity from [`SimulationEngineState`], the
//! open position line, and the engine's round trips (trip construction
//! mirrors the journal pairing shape — `trip-N`, fees ×2 per round trip).
//! Nothing is fetched from an account service and nothing is invented:
//! no trips read *No replay trades yet*, and the position line says flat
//! instead of fabricating a book row.
//!
//! Trip rows read the pairing surface's fields: weighted-average entry
//! (always present) and exit (`None` while nothing closed → `—`), realized
//! PnL (`None` until part of the trip closed → `—`), recorded fees, and
//! the trip status. The exit *reason* column stays off the table — the
//! engine's exit kind (`PositionClosed`) is bookkeeping, not a strategy
//! attribution, and the pairing surface's honest rules about it hold here.

use honba_api_wasm::analytics::{MetricKind, format_metric_value};
use leptos::prelude::*;

use crate::components::simulator::replay_controls::{money_signed, pnl_token};
use crate::stores::simulator::SimulatorStore;

#[component]
pub fn PortfolioPanel(
    /// The simulator store (replay reads).
    store: SimulatorStore,
) -> impl IntoView {
    let memo_store = store.clone();
    let engine_state = Memo::new(move |_| {
        memo_store.replay_generation.track();
        memo_store.replay_state()
    });

    view! {
        <div class="sim-panel sim-portfolio">
            // -- Book summary ------------------------------------------
            <div class="sim-portfolio__books">
                <span class="sim-portfolio__stat">
                    "Initial capital "
                    {move || {
                        format_metric_value(
                            Some(engine_state.get().initial_capital),
                            MetricKind::Currency,
                        )
                    }}
                </span>
                <span class="sim-portfolio__stat">
                    "Realized "
                    <strong class=move || format!(
                        "sim-portfolio__pnl sim-portfolio__pnl--{}",
                        pnl_token(engine_state.get().realized_pnl),
                    )>
                        {move || money_signed(engine_state.get().realized_pnl)}
                    </strong>
                </span>
                <span class="sim-portfolio__stat">
                    "Unrealized "
                    <strong class=move || format!(
                        "sim-portfolio__pnl sim-portfolio__pnl--{}",
                        pnl_token(engine_state.get().unrealized_pnl),
                    )>
                        {move || money_signed(engine_state.get().unrealized_pnl)}
                    </strong>
                </span>
                <span class="sim-portfolio__stat">
                    "Equity "
                    <strong class="sim-portfolio__equity">
                        {move || {
                            format_metric_value(
                                Some(engine_state.get().equity),
                                MetricKind::Currency,
                            )
                        }}
                    </strong>
                </span>
            </div>

            // -- Open position -----------------------------------------
            {move || match &engine_state.get().active_position {
                Some(position) => view! {
                    <div class="sim-portfolio__position">
                        <span class="sim-portfolio__chip">
                            {format!(
                                "{} {} {} @ {} — mark {}, {:+.2}%",
                                position.side.name().to_uppercase(),
                                position.quantity,
                                position.symbol,
                                money_signed(position.entry_price),
                                money_signed(position.current_price),
                                position.unrealized_return_pct * 100.0,
                            )}
                        </span>
                    </div>
                }
                .into_any(),
                None => view! {
                    <div class="sim-portfolio__position">
                        <span class="sim-portfolio__chip sim-portfolio__chip--flat">
                            "Flat (No open position)"
                        </span>
                    </div>
                }
                .into_any(),
            }}

            // -- Round trips -------------------------------------------
            {move || {
                let state = engine_state.get();
                if state.round_trips.is_empty() {
                    view! {
                        <div class="sim-empty">"No replay trades yet."</div>
                    }
                    .into_any()
                } else {
                    let rows = state
                        .round_trips
                        .iter()
                        .map(|trip| {
                            let trip = trip.clone();
                            let id = trip.id.clone();
                            let instrument = trip.instrument.clone();
                            let direction = trip.direction.as_str();
                            let status = trip.status.as_str();
                            let exit = trip
                                .avg_exit
                                .map(money_signed)
                                .unwrap_or_else(|| "—".to_string());
                            let pnl = trip
                                .realized_pnl
                                .map(money_signed)
                                .unwrap_or_else(|| "—".to_string());
                            let pnl_class = if trip.realized_pnl.unwrap_or(0.0) > 0.0 {
                                format!(
                                    "sim-signals__cell sim-num sim-portfolio__pnl sim-portfolio__pnl--{}",
                                    pnl_token(trip.realized_pnl.unwrap_or(0.0)),
                                )
                            } else {
                                "sim-signals__cell sim-num".to_string()
                            };
                            view! {
                                <tr>
                                    <td class="sim-signals__cell">{id}</td>
                                    <td class="sim-signals__cell">{instrument}</td>
                                    <td class="sim-signals__cell">{direction}</td>
                                    <td class="sim-signals__cell sim-num">{trip.quantity}</td>
                                    <td class="sim-signals__cell sim-num">
                                        {money_signed(trip.avg_entry)}
                                    </td>
                                    <td class="sim-signals__cell sim-num">{exit}</td>
                                    <td class=pnl_class>{pnl}</td>
                                    <td class="sim-signals__cell sim-num">
                                        {format_metric_value(Some(trip.fees), MetricKind::Currency)}
                                    </td>
                                    <td class="sim-signals__cell">{status}</td>
                                </tr>
                            }
                        })
                        .collect_view();
                    view! {
                        <table class="sim-signals__table">
                            <thead>
                                <tr>
                                    <th>"Trip"</th>
                                    <th>"Instrument"</th>
                                    <th>"Direction"</th>
                                    <th>"Qty"</th>
                                    <th>"Entry"</th>
                                    <th>"Exit"</th>
                                    <th>"PnL"</th>
                                    <th>"Fees"</th>
                                    <th>"Status"</th>
                                </tr>
                            </thead>
                            <tbody>{rows}</tbody>
                        </table>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
