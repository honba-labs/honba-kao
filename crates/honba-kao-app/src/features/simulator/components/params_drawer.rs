//! The collapsible parameters drawer — the Leptos port of honba-fe's
//! `<aside className="sim-params-drawer">` plus the execution half of its
//! `simulation-controls.tsx` body.
//!
//! Every field edits the existing control panel through
//! [`ControlsStore::patch`](crate::features::simulator::stores::controls_store::ControlsStore::patch);
//! the drawer owns no parallel state. The symbol, timeframe and window
//! already live in the command bar, so the drawer carries what the bar does
//! not: account, benchmark, capital, commission, slippage and fill model,
//! the plan's blocking issues and the run action.
//!
//! Documented deviations from fe:
//! - the account is read-only: the replay books post to the single
//!   sandbox account (`acc-sandbox-primary`) — a picker over one entry would
//!   be a fake affordance;
//! - fe's multi-chart sync box is not ported (the split chart view is the
//!   pending chart-bridge work);
//! - kao's run-lifecycle extras sit under the run button: `Clear run`, and
//!   in debug builds the recorded `Demo fixture`.

use honba_kao_ui_core::icons::outline::{play, sliders, x_close};
use honba_kao_ui_core::{HfButton, HfIconButton, HfInput, HfSelect, SvgIcon};
use leptos::prelude::*;

use crate::features::simulator::kernel::defaults::{FILL_MODEL_DESCRIPTORS, fill_model_descriptor};
use crate::features::simulator::kernel::engine::DEFAULT_ACCOUNT_ID;
use crate::features::simulator::kernel::run::SimRunStatus;
use crate::features::simulator::stores::controls_store::ControlsPatch;
use crate::features::simulator::stores::simulator_store::SimulatorStore;

/// A finite number from an input event, or `None` for an empty / partial
/// entry (the panel keeps its last valid value rather than fe's `Number('')`
/// → `0`).
fn parse_number(event: &web_sys::Event) -> Option<f64> {
    event_target_value(event)
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

/// One numeric panel field (fe `NumberField`).
#[component]
fn NumberField(
    label: &'static str,
    #[prop(into)] value: Signal<f64>,
    #[prop(optional)] suffix: Option<&'static str>,
    on_value: Callback<f64>,
) -> impl IntoView {
    view! {
        <label class="sim-controls__field">
            <span class="sim-controls__label">{label}</span>
            <div class="sim-controls__input-row">
                <HfInput
                    r#type="number"
                    unstyled=true
                    class="sim-controls__input"
                    value=Signal::derive(move || Some(value.get().to_string()))
                    on_input=Callback::new(move |event: web_sys::Event| {
                        if let Some(number) = parse_number(&event) {
                            on_value.run(number);
                        }
                    })
                />
                {suffix.map(|suffix| view! { <span class="sim-controls__suffix">{suffix}</span> })}
            </div>
        </label>
    }
}

#[component]
pub fn ParamsDrawer(
    /// The simulator store (controls, plan, run status).
    store: SimulatorStore,
    /// Starts an execute of the current plan.
    on_run: Callback<()>,
    /// Clears the run and journal.
    on_clear: Callback<()>,
    /// Debug builds: replays the recorded run fixture (`None` in release).
    on_demo: Option<Callback<()>>,
) -> impl IntoView {
    let is_params_open = store.is_params_open;
    let controls = store.controls.clone();
    let panel = controls.panel;
    let name = controls.strategy_name;
    let origin = controls.origin;
    let plan = controls.plan;
    let run = store.run.state;

    let issues = Memo::new(move |_| plan.with(|plan| plan.issues.clone()));
    let busy = Memo::new(move |_| run.with(|state| state.status == SimRunStatus::Running));
    let fill_model = Signal::derive(move || panel.with(|panel| panel.fill_model.clone()));

    let patch = move |patch: ControlsPatch| controls.patch(patch);
    let patch_capital = {
        let patch = patch.clone();
        Callback::new(move |capital| patch(ControlsPatch { capital: Some(capital), ..Default::default() }))
    };
    let patch_commission = {
        let patch = patch.clone();
        Callback::new(move |commission| {
            patch(ControlsPatch { commission: Some(commission), ..Default::default() })
        })
    };
    let patch_slippage = {
        let patch = patch.clone();
        Callback::new(move |slippage| patch(ControlsPatch { slippage: Some(slippage), ..Default::default() }))
    };
    let patch_benchmark = {
        let patch = patch.clone();
        Callback::new(move |event: web_sys::Event| {
            patch(ControlsPatch { benchmark: Some(event_target_value(&event)), ..Default::default() })
        })
    };
    let patch_fill_model = Callback::new(move |event: web_sys::Event| {
        patch(ControlsPatch { fill_model: Some(event_target_value(&event)), ..Default::default() })
    });

    view! {
        <aside class="sim-params-drawer" aria-label="Simulation Parameters">
            <div class="sim-drawer-header">
                <div class="sim-drawer-title-group">
                    <SvgIcon path=sliders() size=14 class="sim-drawer-title-icon" />
                    <span class="sim-drawer-title">"Simulation Parameters"</span>
                </div>
                <HfIconButton
                    unstyled=true
                    class="sim-drawer-close-btn"
                    icon=x_close()
                    size=15
                    title="Close parameters"
                    on_click=Callback::new(move |_| is_params_open.set(false))
                />
            </div>

            <div class="sim-drawer-scroll">
                <div class="sim-controls">
                    <div class="sim-controls__strategy">
                        "Strategy "
                        <strong>{move || name.get()}</strong>
                        " "
                        <span class="sim-controls__origin">
                            {move || format!("({})", origin.get().label())}
                        </span>
                    </div>

                    <label class="sim-controls__field">
                        <span class="sim-controls__label">"Sandboxed Portfolio"</span>
                        <HfInput
                            unstyled=true
                            class="sim-controls__input sim-controls__input--readonly"
                            value=format!("Sandbox Primary ({DEFAULT_ACCOUNT_ID})")
                            disabled=true
                        />
                        <span class="sim-controls__hint">
                            "Replay fills post to the single sandbox account."
                        </span>
                    </label>

                    <label class="sim-controls__field">
                        <span class="sim-controls__label">"Benchmark"</span>
                        <HfInput
                            unstyled=true
                            class="sim-controls__input"
                            value=Signal::derive(move || Some(panel.with(|panel| panel.benchmark.clone())))
                            on_input=patch_benchmark
                        />
                    </label>

                    <NumberField
                        label="Initial capital"
                        value=Signal::derive(move || panel.with(|panel| panel.capital))
                        suffix="INR"
                        on_value=patch_capital
                    />

                    <div class="sim-controls__row">
                        <NumberField
                            label="Commission / fill"
                            value=Signal::derive(move || panel.with(|panel| panel.commission))
                            on_value=patch_commission
                        />
                        <NumberField
                            label="Slippage"
                            value=Signal::derive(move || panel.with(|panel| panel.slippage))
                            suffix="bps"
                            on_value=patch_slippage
                        />
                    </div>

                    <label class="sim-controls__field">
                        <div class="sim-controls__label-row">
                            <span class="sim-controls__label">"Fill model"</span>
                            <span class="sim-controls__badge">
                                {move || {
                                    fill_model_descriptor(&fill_model.get())
                                        .map_or("Execution", |descriptor| descriptor.badge)
                                }}
                            </span>
                        </div>
                        <HfSelect
                            unstyled=true
                            class="sim-controls__input"
                            value=Signal::derive(move || Some(fill_model.get()))
                            on_change=patch_fill_model
                        >
                            {FILL_MODEL_DESCRIPTORS
                                .iter()
                                .map(|descriptor| {
                                    let id = descriptor.id;
                                    view! {
                                        <option value=id selected=move || fill_model.get() == id>
                                            {format!("{} ({})", descriptor.label, descriptor.badge)}
                                        </option>
                                    }
                                })
                                .collect_view()}
                        </HfSelect>
                        <span class="sim-controls__hint">
                            {move || {
                                fill_model_descriptor(&fill_model.get())
                                    .map_or("Simulation execution model.", |descriptor| descriptor.description)
                            }}
                        </span>
                    </label>

                    <Show when=move || !issues.get().is_empty()>
                        <ul class="sim-controls__issues">
                            {move || {
                                issues
                                    .get()
                                    .into_iter()
                                    .map(|issue| view! { <li>{issue}</li> })
                                    .collect_view()
                            }}
                        </ul>
                    </Show>

                    <div class="sim-controls__actions">
                        <HfButton
                            unstyled=true
                            class="sim-run-btn"
                            disabled=Signal::derive(move || busy.get() || !issues.get().is_empty())
                            on_click=Callback::new(move |_| on_run.run(()))
                        >
                            <SvgIcon path=play() size=14 />
                            <span>
                                {move || if busy.get() { "Running Backtest…" } else { "Run Backtest" }}
                            </span>
                        </HfButton>
                        <div class="sim-controls__secondary">
                            <HfButton
                                unstyled=true
                                class="sim-ghost-btn"
                                title="Clear the run and journal"
                                on_click=Callback::new(move |_| on_clear.run(()))
                            >
                                "Clear run"
                            </HfButton>
                            {on_demo.map(|demo| view! {
                                <HfButton
                                    unstyled=true
                                    class="sim-ghost-btn"
                                    title="Debug: replay a recorded completed run"
                                    on_click=Callback::new(move |_| demo.run(()))
                                >
                                    "Demo fixture"
                                </HfButton>
                            })}
                        </div>
                    </div>
                </div>
            </div>
        </aside>
    }
}
