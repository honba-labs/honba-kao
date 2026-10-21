//! The 56px replay command bar — the Leptos port of the `<header
//! className="sim-command-bar">` block in honba-fe `SimulatorView.tsx`.
//!
//! Three zones, in fe's order:
//! - **left** (`.sim-header-left`): strategy select, symbol badge,
//!   grouped timeframe popover, date-range popover;
//! - **center**: the [`ReplayCluster`](super::replay_controls::ReplayCluster)
//!   transport;
//! - **right** (`.sim-header-right`): live P&L chip, confluence pill,
//!   Parameters drawer toggle and the Strategy Studio link.
//!
//! Documented deviations from fe:
//! - the strategy select lists what this build can resolve — the built-in
//!   template plus the active (persisted / designer) selection; fe's
//!   template catalogue and saved library arrive with the studio port (P8);
//! - fe's symbol badge opened the workbench's `SymbolSearchModal`; kao's
//!   modal is bound to the workbench store, so the badge opens a compact
//!   symbol + exchange popover instead of importing across features;
//! - a preset chip only switches `period` (the window then resolves from
//!   today, as kao's plan always has); `Custom` seeds the date inputs from
//!   the current window exactly like fe;
//! - the picker/popover chips (exchange, timeframe, period preset) are
//!   plain reactive `<button>` elements, like fe's own `onClick` chips —
//!   `honba-kao-ui-core` does not yet carry a segmented-control primitive
//!   with a bindable `value`/`on_change`, so this port does not invent one
//!   ahead of that primitive landing; the chip markup/classes still match
//!   fe's (`.sim-tf-chip`, `.sim-preset-chip`) byte for byte.

use honba_kao_ui_core::icons::outline::{
    calendar, chevron_down, clock, external_link, search, sliders, x_close,
};
use honba_kao_ui_core::{HfButton, HfIconButton, SvgIcon, dismiss_on_outside_click};
use leptos::prelude::*;
use leptos_router::components::A;

use crate::features::simulator::components::replay_controls::{
    ReplayCluster, money_signed_grouped,
};
use crate::features::simulator::kernel::controls::PeriodPreset;
use crate::features::simulator::kernel::engine::Indication;
use crate::features::simulator::kernel::engine::controller::SimulationEngineState;
use crate::features::simulator::kernel::selection::{SelectionOrigin, builtin_template};
use crate::features::simulator::stores::controls_store::ControlsPatch;
use crate::features::simulator::stores::simulator_store::SimulatorStore;
use crate::router::paths::ROUTES;

/// fe's three timeframe groups (`MINUTES`, `HOURS`, `DAYS, WEEKS & MONTHS`).
const TIMEFRAME_GROUPS: [(&str, &[&str]); 3] = [
    ("MINUTES", &["1m", "2m", "3m", "5m", "10m", "15m", "30m", "45m"]),
    ("HOURS", &["1h", "2h", "3h", "4h"]),
    ("DAYS, WEEKS & MONTHS", &["1d", "1w", "1mo"]),
];

/// The exchanges the symbol popover offers.
const EXCHANGES: [&str; 2] = ["NSE", "BSE"];

#[component]
pub fn SimCommandBar(
    /// The simulator store.
    store: SimulatorStore,
    /// The page's engine-state memo.
    #[prop(into)]
    engine_state: Signal<SimulationEngineState>,
    /// `SYMBOL.EXCHANGE` the chart and the plan resolve to.
    #[prop(into)]
    instrument_id: Signal<String>,
) -> impl IntoView {
    let is_params_open = store.is_params_open;

    // -- Right zone readouts (fe derives these off `engineState`) -----------
    let live_pnl =
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
    let confluence_counts = Memo::new(move |_| {
        engine_state.with(|state| {
            let count = |indication: Indication| {
                state
                    .active_signals
                    .iter()
                    .filter(|signal| signal.active && signal.indication == indication)
                    .count()
            };
            (count(Indication::Bullish), count(Indication::Bearish))
        })
    });

    view! {
        <header class="sim-command-bar">
            <div class="sim-header-left">
                <StrategySelect store=store.clone() />
                <SymbolBadge store=store.clone() instrument_id=instrument_id />
                <TimeframePicker store=store.clone() />
                <DateRangePicker store=store.clone() />
            </div>

            <ReplayCluster store=store.clone() engine_state=engine_state />

            <div class="sim-header-right">
                <div class="sim-live-pnl-chip" title="Simulated Realized + Unrealized P&L">
                    <span class="sim-pnl-lbl">"P&L:"</span>
                    <span class=move || {
                        if live_pnl.get() >= 0.0 {
                            "sim-pnl-val color-bullish"
                        } else {
                            "sim-pnl-val color-bearish"
                        }
                    }>
                        {move || {
                            let pct = total_return.get() * 100.0;
                            format!(
                                "{} ({}{pct:.2}%)",
                                money_signed_grouped(live_pnl.get()),
                                if pct >= 0.0 { "+" } else { "" },
                            )
                        }}
                    </span>
                </div>

                <div class="sim-confluence-pill" title="Active Signal Confluence">
                    <span class="sim-bull-count">
                        {move || format!("▲ {}", confluence_counts.get().0)}
                    </span>
                    <span class="sim-sep">"/"</span>
                    <span class="sim-bear-count">
                        {move || format!("▼ {}", confluence_counts.get().1)}
                    </span>
                </div>

                <HfButton
                    unstyled=true
                    class="sim-header-btn"
                    title="Configure Strategy & Simulation Parameters"
                    active=is_params_open
                    on_click=Callback::new(move |_| is_params_open.update(|open| *open = !*open))
                >
                    <SvgIcon path=sliders() size=13 />
                    <span>"Parameters"</span>
                </HfButton>

                <A href=ROUTES::STRATEGY attr:class="sim-header-btn sim-header-btn--icon" attr:title="Open in Visual Strategy Studio" attr:aria-label="Open in Visual Strategy Studio">
                    <SvgIcon path=external_link() size=13 />
                </A>
            </div>
        </header>
    }
}

/// fe's grouped strategy `<select>`: the active selection (when it is not
/// the template) and the built-in template.
#[component]
fn StrategySelect(store: SimulatorStore) -> impl IntoView {
    let controls = store.controls.clone();
    let name = controls.strategy_name;
    let origin = controls.origin;
    let (template_graph, template_name) = builtin_template();
    let template_symbol = template_graph
        .universe_explicit
        .first()
        .map(|(symbol, _)| symbol.clone())
        .unwrap_or_else(|| "NSE".to_string());

    let active_group = move || {
        let current = name.get();
        (current != template_name).then(|| {
            let label = match origin.get() {
                SelectionOrigin::Designer => "Active Workspace (From Designer)",
                SelectionOrigin::Library => "User Library (My Saved)",
                SelectionOrigin::Template => "Active Selection",
            };
            view! {
                <optgroup label=label>
                    <option value=current.clone() selected=true>
                        {format!("{current} (Active)")}
                    </option>
                </optgroup>
            }
        })
    };

    view! {
        <div class="sim-select-wrap">
            <span class="sim-field-label">"Strategy"</span>
            <select
                class="sim-header-select"
                aria-label="Strategy"
                on:change=move |event: web_sys::Event| {
                    if event_target_value(&event) == template_name {
                        let (graph, label) = builtin_template();
                        controls.select(graph, label, SelectionOrigin::Template);
                    }
                }
            >
                {active_group}
                <optgroup label="Global Library (Institutional & OpenAlgo)">
                    <option
                        value=template_name
                        selected=move || name.get() == template_name
                    >
                        {format!("{template_name} ({template_symbol})")}
                    </option>
                </optgroup>
            </select>
        </div>
    }
}

/// fe's `.sim-symbol-badge-btn` plus kao's symbol popover (see the module
/// note on why it is not the workbench's search modal).
#[component]
fn SymbolBadge(
    store: SimulatorStore,
    #[prop(into)] instrument_id: Signal<String>,
) -> impl IntoView {
    let panel = store.controls.panel;
    let controls = store.controls.clone();
    let open = RwSignal::new(false);
    let host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(open, host);

    let draft_symbol = RwSignal::new(String::new());
    let draft_exchange = RwSignal::new(String::new());
    let toggle = move |_| {
        if !open.get_untracked() {
            // Seed the draft from the panel; an unchosen symbol starts from
            // the instrument the chart already shows.
            let current = panel.get_untracked();
            let symbol = if current.symbol.trim().is_empty() {
                instrument_id
                    .get_untracked()
                    .rsplit_once('.')
                    .map(|(symbol, _)| symbol.to_string())
                    .unwrap_or_default()
            } else {
                current.symbol.clone()
            };
            draft_symbol.set(symbol);
            draft_exchange.set(current.exchange.clone());
        }
        open.update(|value| *value = !*value);
    };
    let apply = move |_: web_sys::MouseEvent| {
        let symbol = draft_symbol.get_untracked().trim().to_uppercase();
        if symbol.is_empty() {
            return;
        }
        controls.patch(ControlsPatch {
            symbol: Some(symbol),
            exchange: Some(draft_exchange.get_untracked()),
            ..Default::default()
        });
        open.set(false);
    };

    view! {
        <div class="sim-symbol-wrap" node_ref=host>
            <button
                type="button"
                class="sim-symbol-badge-btn"
                title="Change instrument — test this strategy against any other NSE/BSE stock or index"
                on:click=toggle
            >
                <SvgIcon path=search() size=12 class="sim-symbol-search-icon" />
                <span class="sim-symbol-text">{move || instrument_id.get()}</span>
                <span class="sim-symbol-hint">"(Change Symbol)"</span>
                <SvgIcon path=chevron_down() size=11 class="sim-symbol-caret" />
            </button>

            <Show when=move || open.get()>
                <div class="sim-symbol-popover">
                    <span class="sim-tf-group-lbl">"Instrument"</span>
                    <div class="sim-symbol-form">
                        <div class="sim-date-input-field">
                            <span class="sim-date-input-lbl">"Symbol"</span>
                            <input
                                class="sim-date-native-input"
                                placeholder="e.g. RELIANCE"
                                aria-label="Symbol"
                                prop:value=move || draft_symbol.get()
                                on:input=move |event: web_sys::Event| {
                                    draft_symbol.set(event_target_value(&event));
                                }
                            />
                        </div>
                        <div class="sim-preset-chips">
                            {EXCHANGES
                                .iter()
                                .map(|exchange| {
                                    let exchange = *exchange;
                                    view! {
                                        <button
                                            type="button"
                                            class=move || {
                                                if draft_exchange.get() == exchange {
                                                    "sim-preset-chip active".to_string()
                                                } else {
                                                    "sim-preset-chip".to_string()
                                                }
                                            }
                                            on:click=move |_| draft_exchange.set(exchange.to_string())
                                        >
                                            {exchange}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                    <div class="sim-popover-footer">
                        <button
                            type="button"
                            class="sim-apply-btn"
                            disabled=move || draft_symbol.get().trim().is_empty()
                            on:click=apply
                        >
                            "Apply Symbol"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// fe's `.sim-timeframe-wrap`: clock button + the three-row grouped popover.
#[component]
fn TimeframePicker(store: SimulatorStore) -> impl IntoView {
    let panel = store.controls.panel;
    let controls = store.controls.clone();
    let open = RwSignal::new(false);
    let host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(open, host);
    let timeframe = Signal::derive(move || panel.with(|panel| panel.timeframe.clone()));

    let groups = TIMEFRAME_GROUPS
        .iter()
        .map(|(label, frames)| {
            let controls = controls.clone();
            let chips = frames
                .iter()
                .map(|frame| {
                    let frame = *frame;
                    let controls = controls.clone();
                    view! {
                        <button
                            type="button"
                            class=move || {
                                if timeframe.get() == frame {
                                    "sim-tf-chip active".to_string()
                                } else {
                                    "sim-tf-chip".to_string()
                                }
                            }
                            on:click=move |_| {
                                controls.patch(ControlsPatch {
                                    timeframe: Some(frame.to_string()),
                                    ..Default::default()
                                });
                                open.set(false);
                            }
                        >
                            {frame}
                        </button>
                    }
                })
                .collect_view();
            view! {
                <div class="sim-tf-group">
                    <span class="sim-tf-group-lbl">{*label}</span>
                    <div class="sim-tf-chips-row">{chips}</div>
                </div>
            }
        })
        .collect_view();

    view! {
        <div class="sim-timeframe-wrap" node_ref=host>
            <button
                type="button"
                class="sim-timeframe-btn"
                title="Select Timeframe"
                on:click=move |_| open.update(|value| *value = !*value)
            >
                <SvgIcon path=clock() size=12 class="sim-tf-icon" />
                <span class="sim-tf-val">{move || timeframe.get()}</span>
                <SvgIcon path=chevron_down() size=11 class="sim-tf-caret" />
            </button>
            <div class="sim-timeframe-popover" style:display=move || if open.get() { "flex" } else { "none" }>
                {groups}
            </div>
        </div>
    }
}

/// fe's `.sim-date-range-wrap`: calendar button + preset/ISO-date popover.
#[component]
fn DateRangePicker(store: SimulatorStore) -> impl IntoView {
    let panel = store.controls.panel;
    let plan = store.controls.plan;
    let controls = store.controls.clone();
    let open = RwSignal::new(false);
    let host = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside_click(open, host);

    let period = Signal::derive(move || panel.with(|panel| panel.period));
    let range = Memo::new(move |_| plan.with(|plan| plan.range.clone()));

    let on_preset = {
        let controls = controls.clone();
        move |preset: PeriodPreset| {
            if preset == PeriodPreset::Custom {
                let (start, end) = range.get_untracked();
                controls.patch(ControlsPatch {
                    period: Some(PeriodPreset::Custom),
                    start: Some(Some(start)),
                    end: Some(Some(end)),
                    ..Default::default()
                });
            } else {
                controls.patch(ControlsPatch {
                    period: Some(preset),
                    ..Default::default()
                });
            }
        }
    };
    let on_start = {
        let controls = controls.clone();
        move |event: web_sys::Event| {
            let value = event_target_value(&event);
            controls.patch(ControlsPatch {
                period: Some(PeriodPreset::Custom),
                start: Some((!value.is_empty()).then_some(value)),
                end: Some(Some(range.get_untracked().1)),
                ..Default::default()
            });
        }
    };
    let on_end = move |event: web_sys::Event| {
        let value = event_target_value(&event);
        controls.patch(ControlsPatch {
            period: Some(PeriodPreset::Custom),
            start: Some(Some(range.get_untracked().0)),
            end: Some((!value.is_empty()).then_some(value)),
            ..Default::default()
        });
    };

    view! {
        <div class="sim-date-range-wrap" node_ref=host>
            <button
                type="button"
                class="sim-date-range-btn"
                title="Configure Simulation Date Range"
                on:click=move |_| open.update(|value| *value = !*value)
            >
                <SvgIcon path=calendar() size=13 class="sim-date-range-icon" />
                <span class="sim-date-range-preset-pill">{move || period.get().label()}</span>
                <span class="sim-date-range-dates">
                    {move || {
                        let (start, end) = range.get();
                        format!("{start} → {end}")
                    }}
                </span>
                <SvgIcon path=chevron_down() size=11 class="sim-tf-caret" />
            </button>

            <Show when=move || open.get()>
                <div class="sim-date-range-popover">
                    <div class="sim-date-range-header">
                        <div class="sim-date-range-title">
                            <SvgIcon path=calendar() size=13 class="sim-date-range-icon" />
                            <span>"Simulation Date Range"</span>
                        </div>
                        <HfIconButton
                            unstyled=true
                            class="sim-drawer-close-btn"
                            icon=x_close()
                            size=14
                            title="Close"
                            on_click=Callback::new(move |_| open.set(false))
                        />
                    </div>

                    <div class="sim-preset-chips">
                        {PeriodPreset::ALL
                            .iter()
                            .map(|preset| {
                                let preset = *preset;
                                let on_preset = on_preset.clone();
                                view! {
                                    <button
                                        type="button"
                                        class=move || {
                                            if period.get() == preset {
                                                "sim-preset-chip active".to_string()
                                            } else {
                                                "sim-preset-chip".to_string()
                                            }
                                        }
                                        on:click=move |_| on_preset(preset)
                                    >
                                        {preset.label()}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>

                    <div class="sim-date-inputs-row">
                        <div class="sim-date-input-field">
                            <span class="sim-date-input-lbl">"Start Date"</span>
                            <input
                                type="date"
                                class="sim-date-native-input"
                                aria-label="Start Date"
                                prop:value=move || range.get().0
                                on:change=on_start
                            />
                        </div>
                        <div class="sim-date-input-field">
                            <span class="sim-date-input-lbl">"End Date"</span>
                            <input
                                type="date"
                                class="sim-date-native-input"
                                aria-label="End Date"
                                prop:value=move || range.get().1
                                on:change=on_end
                            />
                        </div>
                    </div>

                    <div class="sim-popover-footer">
                        <button
                            type="button"
                            class="sim-apply-btn"
                            on:click=move |_| open.set(false)
                        >
                            "Apply Range"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
