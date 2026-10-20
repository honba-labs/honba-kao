//! The bars container — the Leptos port of honba-fe's
//! `features/simulator/components/interactive-chart.tsx` `SimulatorChart`
//! slice (port phase P6).
//!
//! Loads the bars behind the current selection (`sim::bars`' fold over
//! `GET /bars/{id}`; the deterministic NSE-session preview answers when the
//! store cannot, always labelled as such in the UI — no candle is ever
//! synthesised for a market holiday) and draws the primary candlestick
//! series on the static `openalgo-charts` host, with the same source badge
//! strip fe's toolbar carried.
//!
//! Deliberately **not** ported in this increment (they need bridge/bindings
//! work owned by the chart crate's integration, risk note documented in
//! `sim/mod.rs`):
//! - the multi-pane volume histogram (fe `paneIndex: 1` options are not
//!   in the current `SeriesOptions` binding);
//! - indicator overlays from [`crate::sim::chart_series`] (the engine tier
//!   bindings for `addIndicator` are not exposed by the chart crate yet;
//!   fe attached them through the JS library's registry);
//! - trade markers (the `setMarkers` bridge route is the same pending
//!   bridge work).
//!
//! The chart is useful **before** anything has been simulated: bar loading
//! is keyed to the selection (instrument / window / bar spec), like fe — a
//! cancelled load must not overwrite the newer result, so each effect run
//! carries a generation and an abort.
//!
//! P7 wiring: when a dataset settles (server fold, preview fallback or the
//! no-window reset) the container publishes it through the `on_bars`
//! callback; the store seeds [`ReplayController`] from *that* dataset — fe
//! read the chart's own `dataLayer` bars, so no second fetch happens.
//! While a replay holds the chart, the `replay_bars` signal (visible
//! settled prefix + forming candle) replaces the loaded dataset's slice —
//! fe's `visibleBars` swap; idle falls back to the full loaded list. The
//! state badge and window strip keep describing the *loaded* dataset.
//!
//! Manual serve checklist (the P6 acceptance note, for `make serve`):
//! 1. Open `/simulator` — the strip shows the template selection
//!    (`TCS.NSE`, `5m`, resolved window) and `Ready for simulation`.
//! 2. With the API up, the badge reads `Served bars`; kill the API (or scan
//!    a window the store has no data for) and the badge flips to
//!    `Preview bars` with the labelled note — never silence.
//! 3. Theme switch recolors the canvas without a remount (the host's own
//!    theme handling; no hex is hard-wired on this side).
//! 4. The dock still behaves exactly as P5 left it.
//!
//! Gap, documented per the P6 conventions: the widget theme's
//! `Asia/Kolkata` timezone option travels through the chart crate's
//! `ChartOptions` once it grows a `timezone` field; the x axis reads UTC
//! seconds until then.

use std::sync::Arc;

use leptos::prelude::*;

use openalgo_charts::types::Bar;

use crate::sim::analytics::RunWindow;
use crate::sim::bars::{
    BarsSeam, BarsSource, BarsState, BarsStatus, LoadBarsRequest, load_run_bars_fold,
    no_client_fallback_note, preview_for,
};
use crate::sim::chart_data::ChartBar;
use crate::sim::run::RunAbort;
use crate::sim::selection::StrategyGraphShape;

/// The bar seam as a shared handle (same type the equity card uses).
pub type BarsSeamRef = Arc<dyn BarsSeam + Send + Sync>;

/// `ChartBar` → the engine's bar (identical field set; the engine chart
/// speaks UTC seconds too).
fn chart_bar_to_engine(bar: &ChartBar) -> Bar {
    Bar {
        time: bar.time,
        open: bar.open,
        high: bar.high,
        low: bar.low,
        close: bar.close,
        volume: bar.volume,
    }
}

fn engine_bars(bars: &[ChartBar]) -> Vec<Bar> {
    bars.iter().map(chart_bar_to_engine).collect()
}

/// The source badge copy + CSS modifier (fe `SOURCE_BADGE`, minus its hex
/// fallbacks — tokens only here).
fn source_badge(source: BarsSource) -> Option<(&'static str, &'static str)> {
    match source {
        BarsSource::None => None,
        BarsSource::Server => Some(("Served bars", "server")),
        BarsSource::Preview => Some(("Preview bars", "preview")),
    }
}

/// The honesty strip for the indicator overlays the canvas cannot attach yet
/// (fe drew them through the JS registry; the engine-tier binding is the
/// pending chart-crate work). `Some` when the active graph has indicator
/// nodes at all — silence means nothing is missing.
fn pending_overlays_note(graph: Option<&StrategyGraphShape>) -> Option<String> {
    use crate::sim::chart_series::{OVERLAY_COLORS, graph_specs};
    let specs = graph_specs(graph, OVERLAY_COLORS);
    if specs.is_empty() {
        return None;
    }
    let labels: Vec<&str> = specs.iter().map(|spec| spec.label.as_str()).collect();
    Some(format!(
        "Strategy overlays resolved but not on the canvas yet ({}) — the indicator route lands with the chart-bridge work.",
        labels.join(", ")
    ))
}

#[component]
pub fn SimulatorChart(
    /// `SYMBOL.EXCHANGE` the chart serves.
    #[prop(into)]
    instrument_id: Signal<String>,
    /// The bars window (the plan's range, not the run's — the chart works
    /// before anything was simulated).
    #[prop(into)]
    bars_window: Signal<Option<RunWindow>>,
    /// The timeframe, e.g. `5m`.
    #[prop(into)]
    bar_spec: Signal<String>,
    /// Toolbar readout (`Ready for simulation` / `Run …`).
    #[prop(into)]
    status_note: Signal<String>,
    /// The bar seam; `None` keeps the preview only, labelled.
    bars: Option<BarsSeamRef>,
    /// The active strategy graph, for the overlays-pending honesty strip.
    #[prop(into)]
    strategy: Signal<StrategyGraphShape>,
    /// Publishes each completed dataset to the store, which seeds the P7
    /// replay engine from it (fe read the chart's dataLayer bars — one
    /// fetch, shared). `None` in native builds and unit tests.
    #[prop(into, optional)]
    on_bars: Option<Callback<BarsState>>,
    /// The replay engine's visible-bars marker (`Some(visible)` while a
    /// replay holds the chart: settled bars plus the forming candle;
    /// `None` while idle, when the full loaded dataset shows, fe's hook
    /// fallback).
    #[prop(into, optional)]
    replay_bars: Option<Signal<Option<Vec<ChartBar>>>>,
) -> impl IntoView {
    let bars = StoredValue::new(bars);
    let bars_state = RwSignal::new(BarsState::initial());
    let fetch_abort = StoredValue::<Option<RunAbort>>::new(None);
    let fetch_generation = RwSignal::new(0u64);

    // Bar loading keyed to the selection (fe's keyed `useEffect`); the
    // cancelled-load guard is generation + abort, fe's `active` flag.
    Effect::new(move |_| {
        let instrument = instrument_id.get();
        let window = bars_window.get();
        let spec = bar_spec.get();
        fetch_generation.update(|generation| *generation += 1);
        if let Some(previous) = fetch_abort.get_value() {
            previous.abort();
        }

        let Some(range) = window.clone() else {
            let state = BarsState::initial();
            bars_state.set(state.clone());
            if let Some(publish) = &on_bars {
                publish.run(state);
            }
            return;
        };

        let request_borrow = LoadBarsRequest {
            instrument_id: instrument.clone(),
            from: &range.start,
            to: &range.end,
            bar_spec: &spec,
            anchor: None,
        };

        let Some(seam) = bars.get_value() else {
            let mut state = BarsState::initial();
            state.status = BarsStatus::Ready;
            state.bars = preview_for(&request_borrow);
            state.source = BarsSource::Preview;
            state.note = Some(no_client_fallback_note(&instrument));
            if let Some(publish) = &on_bars {
                publish.run(state.clone());
            }
            bars_state.set(state);
            return;
        };

        bars_state.update(|state| {
            state.status = BarsStatus::Loading;
        });

        let abort = RunAbort::new();
        fetch_abort.set_value(Some(abort.clone()));
        let held_instrument = instrument.clone();
        let held_spec = spec.clone();
        let from = range.start.clone();
        let to = range.end.clone();
        leptos::task::spawn_local(async move {
            let outcome = seam
                .get_bars(&held_instrument, Some(&held_spec), Some(&from), Some(&to))
                .await;
            if abort.is_aborted() {
                return;
            }
            let outcome = outcome.map(|response| response.bars);
            let request = LoadBarsRequest {
                instrument_id: held_instrument,
                from: &from,
                to: &to,
                bar_spec: &held_spec,
                anchor: None,
            };
            let state = load_run_bars_fold(&request, outcome);
            if let Some(publish) = &on_bars {
                publish.run(state.clone());
            }
            bars_state.set(state);
        });
    });

    // -- Derived reads ------------------------------------------------------
    // The replay holds the chart while a visible slice exists (settled
    // prefix + forming candle, fe's `visibleBars` swap); idle falls back to
    // the full loaded dataset — fe's hook returned the raw bars whenever
    // `status === 'idle'`.
    let data_memo = Memo::new(move |_| {
        let visible = replay_bars.as_ref().and_then(|signal| signal.get());
        match visible {
            Some(visible) => engine_bars(&visible),
            None => engine_bars(&bars_state.get().bars),
        }
    });
    let badge = Memo::new(move |_| source_badge(bars_state.get().source));
    let note = Memo::new(move |_| bars_state.get().note);
    let window_line = Memo::new(move |_| {
        bars_window
            .get()
            .map(|range| format!("{} → {}", range.start, range.end))
            .unwrap_or_default()
    });
    let overlays_note =
        Memo::new(move |_| strategy.with(|graph| pending_overlays_note(Some(graph))));

    view! {
        <div class="sim-chart">
            <div class="sim-chart__strip">
                <span class="sim-chart__symbol">{move || instrument_id.get()}</span>
                <span class="sim-chart__tf">{move || bar_spec.get()}</span>
                <span class="sim-chart__window">{move || window_line.get()}</span>
                <span class="sim-chart__status">{move || status_note.get()}</span>
                {move || {
                    badge.get().map(|(label, modifier)| {
                        view! {
                            <span class=format!("sim-chart__badge sim-chart__badge--{modifier}")>
                                {label}
                            </span>
                        }
                    })
                }}
            </div>
            <div class="sim-chart__chart">
                <crate::components::charts::OpenAlgoChart
                    chart_mode="candlestick".to_string()
                    data=data_memo.into()
                    class="openalgo-chart"
                    label="Simulation chart"
                />
            </div>
            <Show when=move || note.get().is_some()>
                <div class="sim-chart__note">{move || note.get().unwrap_or_default()}</div>
            </Show>
            <Show when=move || overlays_note.get().is_some()>
                <div class="sim-chart__note">
                    {move || overlays_note.get().unwrap_or_default()}
                </div>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::selection::builtin_template;

    #[test]
    fn chart_bars_map_field_for_field_onto_the_engine_bar() {
        let bar = chart_bar_to_engine(&ChartBar {
            time: 1_704_153_600,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.7,
            volume: 900.0,
        });
        assert_eq!(bar.time, 1_704_153_600);
        assert_eq!(bar.open, 1.0);
        assert_eq!(bar.high, 2.0);
        assert_eq!(bar.low, 0.5);
        assert_eq!(bar.close, 1.7);
        assert_eq!(bar.volume, 900.0);
        // An empty list stays empty — no candle is ever synthesised.
        assert!(engine_bars(&[]).is_empty());
    }

    #[test]
    fn the_source_badge_labels_the_source_never_the_other_way_round() {
        assert_eq!(source_badge(BarsSource::None), None);
        assert_eq!(
            source_badge(BarsSource::Server),
            Some(("Served bars", "server"))
        );
        assert_eq!(
            source_badge(BarsSource::Preview),
            Some(("Preview bars", "preview"))
        );
    }

    #[test]
    fn the_overlays_note_names_resolved_specs_only_when_the_graph_has_them() {
        let (graph, _) = builtin_template();
        let note = pending_overlays_note(Some(&graph)).expect("the template has two SMAs");
        assert!(note.contains("Fast SMA"));
        assert!(note.contains("Slow SMA"));
        // A node-less graph stays silent — no noise about overlays that do
        // not exist.
        assert_eq!(pending_overlays_note(None), None);
        let mut bare = builtin_template().0;
        bare.nodes.clear();
        assert_eq!(pending_overlays_note(Some(&bare)), None);
    }
}
