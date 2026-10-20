//! The signals dock panel — the Leptos port of honba-fe `signals`
//! (the replay engine's confluence data, P7).
//!
//! fe surfaced the engine's `activeSignals` around its own chart gadgets;
//! as pinned in the port doc, the dock tab holds the rows rather than a
//! floating panel. A bull/bear confluence summary sits in the header, then
//! one row per close-phase condition: label, fired pill, indication, the
//! 2dp indicator and threshold values, and the detail line.
//!
//! Honesty:
//! - a row with no measured value shows `—` (fe left that slot blank);
//! - before any bar has closed there is nothing to tabulate — the empty
//!   state says so rather than rendering a zero table;
//! - a bar that closed with nothing configured states *that bar's* fact.
//!
//! Rows re-render per replay generation; every class string is computed
//! once per snapshot (the memo re-runs on the store's generation bump), so
//! no leak closures are needed inside the table.

use leptos::prelude::*;

use crate::stores::simulator::SimulatorStore;

#[component]
pub fn SignalsPanel(
    /// The simulator store (replay reads).
    store: SimulatorStore,
) -> impl IntoView {
    let memo_store = store.clone();
    let engine_state = Memo::new(move |_| {
        memo_store.replay_generation.track();
        memo_store.replay_state()
    });

    let confluence = Memo::new(move |_| engine_state.get().last_confluence);

    view! {
        <div class="sim-panel sim-signals">
            {move || {
                // Header confluence pill (fe's bull/bear badge).
                confluence
                    .get()
                    .map(|snapshot| {
                        let flavour = if snapshot.bullish_score > snapshot.bearish_score
                            && snapshot.bullish_score > 0.0
                        {
                            "buy"
                        } else if snapshot.bearish_score > snapshot.bullish_score
                            && snapshot.bearish_score > 0.0
                        {
                            "sell"
                        } else {
                            "hold"
                        };
                        let dominant = snapshot.bullish_score.max(snapshot.bearish_score);
                        view! {
                            <div class="sim-signals__head">
                                <span class=format!("sim-signals__pill sim-signals__pill--{flavour}")>
                                    {match flavour {
                                        "buy" => "BULLISH",
                                        "sell" => "BEARISH",
                                        _ => "HOLD",
                                    }}
                                </span>
                                <span class="sim-signals__ratio">
                                    {format!(
                                        "Confluence {dominant}/{} ({:.0}%)",
                                        snapshot.total_signals,
                                        snapshot.confluence_ratio * 100.0,
                                    )}
                                </span>
                                <span class="sim-signals__stamp">
                                    {format!("{} @ {}", snapshot.time_str, snapshot.price)}
                                </span>
                            </div>
                        }
                    })
            }}

            {move || {
                let state = engine_state.get();
                if state.active_signals.is_empty() {
                    let copy = if state.total_bars == 0 {
                        "No bars are loaded for the replay yet."
                    } else if state.last_confluence.is_none() {
                        "No bar has closed yet — the confluence rows appear on the first close."
                    } else {
                        "This bar closed with nothing configured to fire."
                    };
                    view! { <div class="sim-empty">{copy}</div> }.into_any()
                } else {
                    let rows = state
                        .active_signals
                        .iter()
                        .map(|signal| {
                            let row_class = if signal.active {
                                "sim-signals__row sim-signals__row--fired"
                            } else {
                                "sim-signals__row"
                            };
                            let fired = if signal.active { "fired" } else { "—" };
                            let ind_class = format!(
                                "sim-signals__cell sim-signals__ind sim-signals__ind--{}",
                                signal.indication.name(),
                            );
                            let indication = signal.indication.name();
                            let value = format_option(signal.indicator_value);
                            let threshold = format_option(signal.threshold_value);
                            view! {
                                <tr class=row_class>
                                    <td class="sim-signals__cell">{signal.label.clone()}</td>
                                    <td class="sim-signals__cell">{fired}</td>
                                    <td class=ind_class>{indication}</td>
                                    <td class="sim-signals__cell sim-num">{value}</td>
                                    <td class="sim-signals__cell sim-num">{threshold}</td>
                                    <td class="sim-signals__cell sim-signals__details">
                                        {signal.details.clone()}
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view();
                    view! {
                        <table class="sim-signals__table">
                            <thead>
                                <tr>
                                    <th>"Label"</th>
                                    <th>"State"</th>
                                    <th>"Indication"</th>
                                    <th>"Value"</th>
                                    <th>"Threshold"</th>
                                    <th>"Detail"</th>
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

/// fe left a missing value as an empty string; a visible `—` states the
/// absence the same way the other honesty surfaces do.
fn format_option(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "—".to_string())
}
