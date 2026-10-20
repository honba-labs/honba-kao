//! The equity & drawdown card — the Leptos port of honba-fe
//! `features/simulator/components/equity-curve-card.tsx` (port phase P6).
//!
//! The curve is the **realised** equity from the journal. The run contract
//! has no equity series and no benchmark, so the benchmark overlay is
//! attempted from the bar seam over the same run window and simply omitted —
//! with the reason labelled — when it cannot be drawn (risk R5, honesty
//! invariant: an overlay is never fabricated).
//!
//! Documented deviations from fe:
//! - fe reused its native SVG `EquityDrawdownChart`; on kao the curve rides
//!   the same static `openalgo-charts` host the candle chart uses, so the
//!   equity points become pseudo-bars (`open=high=low=close`, volume 0) on a
//!   daily timestamp grid built from the run window — or an index-based
//!   synthetic grid when the window is unknown (the x axis then reads the
//!   epoch day count; the window is usually known because a run built it).
//!   The benchmark overlay rides the same timestamps, so both series stay
//!   index-aligned.
//! - fe's `hasBars` fallback mislabeled a missing client as "needs a daily
//!   equity curve"; this port says what is actually missing.
//! - The drawdown is shown as a stat line (`max_drawdown_pct` from
//!   `sim::ops`), not a shaded second pane — the static host has no dual
//!   pane for it yet.

use std::sync::Arc;

use honba_api_wasm::analytics::{
    benchmark_equity, benchmark_instrument_id, format_metric_value, AnalyticsInput, ClosePoint,
    EquitySampling, EquitySeries, MetricKind, RunWindow,
};
use leptos::prelude::*;

use openalgo_charts::types::{Bar, SeriesKind, SeriesOptions};

use crate::features::simulator::kernel::bars::{seconds_of_date, BarsSeam};
use crate::features::simulator::kernel::chart_data::to_chart_bars;
use crate::features::simulator::kernel::ops::max_drawdown_pct;
use crate::features::simulator::kernel::run::RunAbort;

/// The bar seam as a shared handle (wasm only — the native build never
/// renders this page).
pub type BarsSeamRef = Arc<dyn BarsSeam + Send + Sync>;

/// The day-step of the daily axis: one pseudo-bar per day.
const DAY_SECONDS: i64 = 86_400;

/// Equity values → pseudo bars on a daily grid (`EquityDrawdownChart`'s
/// input, when the chart host has to carry it). Timestamps step
/// [`DAY_SECONDS`] from the run window's start when known, else from the
/// epoch — monotonic either way.
fn equity_bars(values: &[f64], start_date: Option<&str>) -> Vec<Bar> {
    let base = start_date
        .filter(|date| !date.is_empty())
        .map(seconds_of_date)
        .unwrap_or(0);
    values
        .iter()
        .enumerate()
        .map(|(index, &value)| Bar {
            time: base + index as i64 * DAY_SECONDS,
            open: value,
            high: value,
            low: value,
            close: value,
            volume: 0.0,
        })
        .collect()
}

/// The benchmark overlay's line-colour token (`--brand`); the chart host
/// resolves CSS variable names in-page.
const OVERLAY_COLOR: &str = "--brand";

#[component]
pub fn EquityCard(
    /// Realised equity (fe `EquityCurveCardProps.equity`).
    #[prop(into)]
    equity: Signal<EquitySeries>,
    /// The resolved run window (plan range), `None` until a run was planned.
    #[prop(into)]
    window: Signal<Option<RunWindow>>,
    /// The benchmark label (`controls.benchmark`), e.g. "NIFTY 50".
    #[prop(into)]
    benchmark_label: Signal<String>,
    /// The run's `bar_spec`, the timeframe the benchmark is fetched on.
    #[prop(into)]
    bar_spec: Signal<String>,
    /// Fresh per run so a previous run's overlay is dropped (fe `runId`).
    #[prop(into)]
    run_generation: Signal<u64>,
    /// The bar seam; `None` keeps the curve alone with a labelled note.
    bars: Option<BarsSeamRef>,
) -> impl IntoView {
    let bars = StoredValue::new(bars);
    let benchmark = RwSignal::new(None::<Vec<f64>>);
    let note = RwSignal::new(None::<String>);

    // -- Derived reads ------------------------------------------------------
    let sampling_line = Memo::new(move |_| match equity.get().sampling {
        EquitySampling::Daily => "Realised equity, daily from the journal",
        EquitySampling::Trade => "Realised equity per closed round trip",
    });
    let money_line = Memo::new(move |_| {
        let series = equity.get();
        format!(
            "₹{} → ₹{}",
            format_metric_value(Some(series.initial), MetricKind::Currency),
            format_metric_value(Some(series.final_value), MetricKind::Currency),
        )
    });
    let drawdown_line = Memo::new(move |_| {
        let max_dd = max_drawdown_pct(&equity.get().equity);
        if max_dd > 0.0 {
            format!("Max drawdown −{max_dd:.2}%")
        } else {
            String::new()
        }
    });
    let is_empty = Memo::new(move |_| equity.get().equity.len() < 2);

    // The primary curve rides the chart's reactive `data` signal.
    let data_memo = Memo::new(move |_| {
        let series = equity.get();
        let held_window = window.get();
        let start = held_window.as_ref().map(|w| w.start.as_str());
        equity_bars(&series.equity, start)
    });

    // The benchmark overlay's bars, when the fetch produced an index-aligned
    // series.
    let overlay_bars = Memo::new(move |_| {
        benchmark
            .get()
            .map(|values| {
                let held_window = window.get();
                let start = held_window.as_ref().map(|w| w.start.as_str());
                equity_bars(&values, start)
            })
    });

    // The benchmark overlay fetch (fe's `useEffect`). Keyed on exactly what
    // the request depends on plus the run generation; a `spawn_local` future
    // answers through abort guards so a superseded run's answer is dropped.
    let fetch_abort = StoredValue::<Option<RunAbort>>::new(None);
    Effect::new(move |_| {
        // The fresh-run identity (fe `runId`): every run re-keys the fetch.
        let _generation = run_generation.get();
        let window_value = window.get();
        let equity = equity.get();
        let label = benchmark_label.get();
        let spec = bar_spec.get();

        benchmark.set(None);
        if let Some(previous) = fetch_abort.get_value() {
            previous.abort();
        }

        let Some(range) = window_value else {
            note.set(Some("Benchmark overlay needs a run window.".to_string()));
            return;
        };
        let Some(instrument_id) = benchmark_instrument_id(Some(&label), None) else {
            note.set(Some("Benchmark overlay needs a benchmark symbol.".to_string()));
            return;
        };
        if equity.sampling != EquitySampling::Daily {
            note.set(Some("Benchmark overlay needs a daily equity curve.".to_string()));
            return;
        }
        let Some(seam) = bars.get_value() else {
            // fe mislabeled this path "needs a daily equity curve"; the port
            // says what is actually missing.
            note.set(Some(
                "Benchmark bars unavailable (no bar client); the run curve is shown alone."
                    .to_string(),
            ));
            return;
        };

        let abort = RunAbort::new();
        fetch_abort.set_value(Some(abort.clone()));
        let held_equity = equity;
        let held_id = instrument_id.clone();
        let held_label = label;
        let held_spec = spec;
        let from = range.start.clone();
        let to = range.end.clone();
        let window_owned = range;
        leptos::task::spawn_local(async move {
            let outcome = seam
                .get_bars(&held_id, Some(&held_spec), Some(&from), Some(&to))
                .await;
            if abort.is_aborted() {
                return;
            }
            match outcome {
                Ok(response) => {
                    let closes: Vec<ClosePoint> = to_chart_bars(&response.bars)
                        .iter()
                        .map(|bar| ClosePoint {
                            time: bar.time,
                            close: bar.close,
                        })
                        .collect();
                    let input = AnalyticsInput {
                        metrics: None,
                        trips: &[],
                        window: Some(&window_owned),
                        initial_capital: None,
                    };
                    match benchmark_equity(&closes, &held_equity, &input) {
                        Some(series) => {
                            benchmark.set(Some(series));
                            note.set(Some(format!(
                                "Benchmark overlay: {held_label} ({held_id}), buy-and-hold normalised to the starting capital."
                            )));
                        }
                        None => note.set(Some(format!(
                            "No {held_spec} benchmark bars for {held_id} in this window; the run curve is shown alone."
                        ))),
                    }
                }
                Err(error) => {
                    // fe: `controller.signal.aborted` answers are dropped.
                    if abort.is_aborted() {
                        return;
                    }
                    note.set(Some(format!(
                        "Benchmark bars unavailable ({error}); the run curve is shown alone."
                    )));
                }
            }
        });
    });

    // The overlay series lives on the mounted chart handle; the primary
    // curve is the host's reactive `data`. Re-created on each benchmark
    // repaint so the previous run's line never lingers.
    let chart_handle = RwSignal::new(None::<openalgo_charts::chart::ChartHandle>);
    let overlay_handle = StoredValue::<Option<openalgo_charts::chart::SeriesHandle>>::new(None);
    Effect::new(move |_| {
        let Some(handle) = chart_handle.get() else { return };
        match overlay_bars.get() {
            Some(bars) => {
                overlay_handle.update_value(|slot| {
                    if let Some(previous) = slot.take() {
                        previous.remove();
                    }
                });
                let options = SeriesOptions {
                    color: Some(OVERLAY_COLOR.to_string()),
                    line_width: Some(1.5),
                    ..Default::default()
                };
                if let Ok(series) = handle.add_series(SeriesKind::Line, Some(&options)) {
                    let _ = series.set_data(&bars);
                    overlay_handle.set_value(Some(series));
                }
                handle.fit_content();
            }
            None => {
                overlay_handle.update_value(|slot| {
                    if let Some(previous) = slot.take() {
                        previous.remove();
                    }
                });
            }
        }
    });

    view! {
        <div class="sim-equity">
            <div class="sim-equity__head">
                <span class="sim-equity__caption">{move || sampling_line.get()}</span>
                <span class="sim-equity__money">{move || money_line.get()}</span>
            </div>
            <Show
                when=move || !is_empty.get()
                fallback=move || {
                    view! {
                        <div class="sim-equity__empty">
                            "No closed round trips to draw yet."
                        </div>
                    }
                }
            >
                <div class="sim-equity__chart">
                    <crate::shared::components::charts::OpenAlgoChart
                        chart_mode="area".to_string()
                        data=data_memo.into()
                        class="openalgo-chart"
                        label="Equity curve"
                        chart_handle=chart_handle
                    />
                </div>
                <div class="sim-equity__drawdown">{move || drawdown_line.get()}</div>
            </Show>
            {move || {
                note.get().map(|note| {
                    view! { <div class="sim-equity__note">{note}</div> }
                })
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openalgo_charts::types::ChartSeries;

    // `ChartSeries::line` requires the bar list to keep timestamps; the
    // pseudo-bars carry equity values only, so a smoke check on the ctor.
    #[test]
    fn the_chart_series_ctor_is_bar_shaped() {
        let bars = equity_bars(&[1.0, 3.0, 2.5], Some("2024-01-02"));
        let series = ChartSeries::area(bars);
        assert_eq!(series.bars.len(), 3);
    }

    #[test]
    fn equity_bars_step_one_day_from_the_window_start_when_known() {
        let bars = equity_bars(&[1.0, 3.0, 2.5], Some("2024-01-02"));
        assert_eq!(bars.len(), 3);
        // `seconds_of_date("2024-01-02")` = 1704153600.
        assert_eq!(bars[0].time, 1_704_153_600);
        assert_eq!(bars[1].time, bars[0].time + 86_400);
        assert_eq!(bars[2].high, 2.5);
        assert!(bars.windows(2).all(|pair| pair[0].time < pair[1].time));
        // Every series point is a flat pseudo-bar with no volume.
        for bar in &bars {
            assert_eq!(bar.open, bar.close);
            assert_eq!(bar.high, bar.close);
            assert_eq!(bar.low, bar.close);
            assert_eq!(bar.volume, 0.0);
        }
    }

    #[test]
    fn an_unknown_window_falls_back_to_a_monotonic_synthetic_axis() {
        let bars = equity_bars(&[5.0, 6.0], None);
        assert_eq!(bars[0].time, 0);
        assert_eq!(bars[1].time, 86_400);
        // Malformed dates behave like no window at all.
        let bars = equity_bars(&[5.0], Some("not-a-date"));
        assert_eq!(bars[0].time, 0);
    }
}
