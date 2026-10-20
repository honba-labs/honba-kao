//! The replay toolbar — the Leptos port of honba-fe
//! `features/simulator/components/replay-toolbar.tsx` (P7).
//!
//! fe's rows, in order: transport + speed pills + multi-timeframe toggle;
//! the live signal HUD; the sandboxed portfolio ticker; a scrubber. This
//! port carries all of them except:
//!
//! - the **multi-chart toggle** — the split-view chart is the pending
//!   bridge-crate work (volume pane / second series binding), and a toggle
//!   that flips state without rendering anything would be a fake
//!   affordance; it returns with the second chart instead;
//! - the equity readout shows the store's currency shape (two decimals,
//!   grouped) while fe used `toLocaleString(maximumFractionDigits: 0)` —
//!   an integer-only cosmetic difference, noted here rather than re-sized.
//!
//! "Waiting for bars…": fe showed it whenever `activeSignals` was empty —
//! even on a graph with no conditions at all. Here it means "no bar has
//! closed yet"; a closed bar with nothing configured renders no HUD rows
//! (the distinctive copy lives on the signals tab).
//!
//! Reactivity: the component owns no controller — it reads a memo keyed on
//! the store's replay generation and calls the store's transport methods;
//! the driver loop, token guards and visible-bar emission all live in the
//! store (see `stores/simulator.rs`).
//!
//! Progress percentage: fe computed `(index / (total - 1)) * 100`, which
//! renders `NaN%` on a one-bar dataset; here a single bar reads `0%`
//! (`total <= 1` guards the division) — pinned, deliberate.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::sim::engine::Indication;
use crate::sim::engine::ReplayStatus;
use crate::sim::engine::controller::SimulationEngineState;
use crate::sim::engine::evaluator::{ConfluenceSnapshot, SignalEvaluation};
use crate::stores::simulator::SimulatorStore;

/// The fe toolbar's speed pill set (`const SPEEDS`, verbatim).
const SPEEDS: [f64; 5] = [1.0, 5.0, 10.0, 25.0, 50.0];

/// `₹` + sign + two decimals (fe `₹${pnl >= 0 ? '+' : ''}${pnl.toFixed(2)}`).
pub(crate) fn money_signed(value: f64) -> String {
    let sign = if value < 0.0 { "−" } else { "+" };
    format!("₹{sign}{:.2}", value.abs())
}

/// The pill colour token a PnL reads as.
pub(crate) fn pnl_token(value: f64) -> &'static str {
    if value > 0.0 {
        "gain"
    } else if value < 0.0 {
        "loss"
    } else {
        "flat"
    }
}

/// fe's equity readout shape: `toLocaleString(maximumFractionDigits: 0)`
/// — grouped integer rupees.
fn money_grouped_int(value: f64) -> String {
    let whole = value.round() as i64;
    let digits = whole.abs().to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if whole < 0 {
        format!("₹−{grouped}")
    } else {
        format!("₹{grouped}")
    }
}

#[component]
pub fn ReplayControls(
    /// The simulator store — every transport method plus the generation
    /// signal the engine-state memo tracks.
    store: SimulatorStore,
) -> impl IntoView {
    let memo_store = store.clone();
    let engine_state = Memo::new(move |_| {
        memo_store.replay_generation.track();
        memo_store.replay_state()
    });

    let status_line = Memo::new(move |_| engine_state.get().status);
    let has_bars = Memo::new(move |_| engine_state.get().total_bars > 0);
    let progress = Memo::new(move |_| {
        let state = engine_state.get();
        if state.total_bars > 1 {
            (state.current_bar_index as f64) / (state.total_bars - 1) as f64 * 100.0
        } else {
            0.0
        }
    });
    let scrub_max = Memo::new(move |_| engine_state.get().total_bars.saturating_sub(1));
    let bar_line = Memo::new(move |_| {
        let state = engine_state.get();
        format!(
            "Bar {} / {} ({:.0}%)",
            state.current_bar_index + 1,
            state.total_bars,
            progress.get()
        )
    });

    // -- Transport ----------------------------------------------------------
    let transport_store = store.clone();
    let on_play_pause = move |_| {
        if status_line.get() == ReplayStatus::Playing {
            transport_store.replay_pause();
        } else {
            transport_store.replay_play();
        }
    };
    let on_step = {
        let store = store.clone();
        move |_| store.replay_step()
    };
    let on_reset = {
        let store = store.clone();
        move |_| store.replay_reset()
    };
    let on_instant = {
        let store = store.clone();
        move |_| store.replay_instant()
    };
    let button_label = Memo::new(move |_| match engine_state.get().status {
        ReplayStatus::Playing => "Pause".to_string(),
        ReplayStatus::Completed => "Replay".to_string(),
        _ => "Play".to_string(),
    });
    let is_playing = Memo::new(move |_| status_line.get() == ReplayStatus::Playing);

    // -- Scrubber ----------------------------------------------------------
    let scrub_store = store.clone();
    let on_scrub = move |event: leptos::ev::Event| {
        let input = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok());
        let Some(input) = input else {
            return;
        };
        if let Ok(value) = input.value().parse::<usize>() {
            scrub_store.replay_jump(value);
        }
    };

    view! {
        <div class="sim-replay">
            // -- Row 1: transport + speed --------------------------------
            <div class="sim-replay__row">
                <button
                    type="button"
                    class="sim-replay__btn"
                    title="Replay the loaded bars tick by tick"
                    disabled=move || !has_bars.get()
                    on:click=on_play_pause
                >
                    {move || button_label.get()}
                </button>
                <button
                    type="button"
                    class="sim-replay__btn"
                    title="Step 1 Bar forward"
                    disabled=move || !has_bars.get() || is_playing.get()
                    on:click=on_step
                >
                    "Step"
                </button>
                <button
                    type="button"
                    class="sim-replay__btn"
                    title="Rewind to start"
                    disabled=move || !has_bars.get()
                    on:click=on_reset
                >
                    "Reset"
                </button>
                <button
                    type="button"
                    class="sim-replay__btn sim-replay__btn--instant"
                    title="Fast simulate entire dataset to the end"
                    disabled=move || !has_bars.get()
                    on:click=on_instant
                >
                    "Run to End"
                </button>

                <div class="sim-replay__speeds">
                    <span class="sim-replay__speed-label">"Speed:"</span>
                    {SPEEDS
                        .iter()
                        .map(|speed| {
                            let value = *speed;
                            let store = store.clone();
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        let state = engine_state.get();
                                        if state.speed == value {
                                            "sim-replay__btn sim-replay__btn--active".to_string()
                                        } else {
                                            "sim-replay__btn".to_string()
                                        }
                                    }
                                    on:click=move |_| store.replay_set_speed(value)
                                >
                                    {format!("{value}x")}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>

            // -- Row 2: live signal HUD ----------------------------------
            <div class="sim-replay__row sim-replay__row--hud">
                <span class="sim-replay__speed-label">"Signals:"</span>
                {move || {
                    let state = engine_state.get();
                    match signal_hud(&state) {
                        Huds::Waiting => view! {
                            <span class="sim-replay__hud-empty">"Waiting for bars…"</span>
                        }
                        .into_any(),
                        Huds::Rows(rows) => rows
                            .into_iter()
                            .map(|pill| {
                                view! {
                                    <span
                                        class=pill.class
                                        title=pill.details
                                    >
                                        {pill.label}
                                    </span>
                                }
                            })
                            .collect_view()
                            .into_any(),
                    }
                }}
                {move || {
                    engine_state
                        .get()
                        .last_confluence
                        .map(|confluence| {
                            let classes = match confluence_action(&confluence) {
                                Some("buy") => "sim-replay__confluence sim-replay__confluence--buy".to_string(),
                                Some("sell") => "sim-replay__confluence sim-replay__confluence--sell".to_string(),
                                _ => "sim-replay__confluence".to_string(),
                            };
                            view! {
                                <span class=classes>
                                    {format!(
                                        "Confluence: {}/{}",
                                        formatted_score(confluence.bullish_score),
                                        confluence.total_signals,
                                    )}
                                </span>
                            }
                        })
                }}
            </div>

            // -- Row 3: sandbox book ticker ------------------------------
            <div class="sim-replay__row sim-replay__row--book">
                {move || {
                    let state = engine_state.get();
                    match &state.active_position {
                        Some(position) => view! {
                            <span class="sim-replay__position">
                                {format!(
                                    "{} {} {} @ {}",
                                    position.side.name().to_uppercase(),
                                    position.quantity,
                                    position.symbol,
                                    money_signed(position.entry_price),
                                )}
                            </span>
                        }
                        .into_any(),
                        None => view! {
                            <span class="sim-replay__hud-empty">
                                "Flat (No open position)"
                            </span>
                        }
                        .into_any(),
                    }
                }}
                <span class="sim-replay__pnl">
                    "Unrealized: "
                    <strong class=move || format!("sim-replay__pnl--{}", pnl_token(engine_state.get().unrealized_pnl))>
                        {move || money_signed(engine_state.get().unrealized_pnl)}
                    </strong>
                </span>
                <span class="sim-replay__pnl">
                    "Realized: "
                    <strong class=move || format!("sim-replay__pnl--{}", pnl_token(engine_state.get().realized_pnl))>
                        {move || money_signed(engine_state.get().realized_pnl)}
                    </strong>
                </span>
                <span class="sim-replay__pnl">
                    "Equity: "
                    <strong class="sim-replay__equity">
                        {move || money_grouped_int(engine_state.get().equity)}
                    </strong>
                </span>
            </div>

            // -- Row 4: scrubber ----------------------------------------
            <div class="sim-replay__row sim-replay__row--scrub">
                <input
                    type="range"
                    min="0"
                    max=move || scrub_max.get().to_string()
                    value=move || engine_state.get().current_bar_index.to_string()
                    disabled=move || !has_bars.get()
                    on:change=on_scrub
                    class="sim-replay__scrub"
                    title="Scrub through the replay"
                />
                <span class="sim-replay__bar-line">{move || bar_line.get()}</span>
            </div>
        </div>
    }
}

/// The HUD row's shape: nothing closed yet, or the per-condition pills.
enum Huds {
    /// fe's "Waiting for bars…" — no close-phase evaluation has run.
    Waiting,
    Rows(Vec<HudPill>),
}

/// One signal pill (fe's activeSignals map row).
struct HudPill {
    /// The label + arrow/branch text.
    label: String,
    /// Details tooltip copy.
    details: String,
    /// CSS class including the fired/unfired modifier.
    class: String,
}

/// fe's HUD logic — per-condition pills, plus the waiting state.
fn signal_hud(state: &SimulationEngineState) -> Huds {
    if state.current_tick.is_none() && state.active_signals.is_empty() {
        return Huds::Waiting;
    }
    if state.active_signals.is_empty() {
        // A bar has closed and nothing is configured: render nothing rather
        // than resurrecting the "waiting" copy (the empty condition is on
        // the signals tab).
        return Huds::Rows(Vec::new());
    }
    let rows = state
        .active_signals
        .iter()
        .map(|signal| HudPill {
            label: hud_label(signal),
            details: signal.details.clone(),
            class: hud_class(signal),
        })
        .collect();
    Huds::Rows(rows)
}

fn hud_label(signal: &SignalEvaluation) -> String {
    if !signal.active {
        return format!("{}: —", signal.label);
    }
    match signal.indication {
        Indication::Bullish => format!("{}: ▲ BUY", signal.label),
        _ => format!("{}: ▼ SELL", signal.label),
    }
}

fn hud_class(signal: &SignalEvaluation) -> String {
    if !signal.active {
        return "sim-replay__hud sim-replay__hud--idle".to_string();
    }
    match signal.indication {
        Indication::Bullish => "sim-replay__hud sim-replay__hud--buy".to_string(),
        _ => "sim-replay__hud sim-replay__hud--sell".to_string(),
    }
}

/// The confluence pill's colour class — `buy`/`sell` from the snapshot's
/// score comparison (fe coloured by `action === 'buy'` / `'sell'`),
/// `None` when nothing fired (a hold pill renders uncoloured in fe).
fn confluence_action(confluence: &ConfluenceSnapshot) -> Option<&'static str> {
    if confluence.bullish_score > confluence.bearish_score && confluence.bullish_score > 0.0 {
        Some("buy")
    } else if confluence.bearish_score > confluence.bullish_score && confluence.bearish_score > 0.0
    {
        Some("sell")
    } else {
        None
    }
}

/// fe interpolated the raw number (`{confluence.bullishScore}`) — a bare
/// score, `{:.2}` would read `1.00` where fe read `1`.
fn formatted_score(score: f64) -> String {
    format!("{score}")
}
