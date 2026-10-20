//! The metrics ribbon — the Leptos port of honba-fe
//! `features/simulator/components/metrics-ribbon.tsx` (port phase P5).
//!
//! Echoes the run payload's wire metrics exactly and shows the *derived*
//! ones (CAGR, Calmar, win rate, profit factor, exposure) tagged as
//! derived, plus an explicit `n/a` for the metrics the contract cannot
//! supply. The provenance chip and the tooltip make the difference between
//! a reported and a computed number visible at a glance (risk R5, honesty
//! invariant 1: metrics are never invented).

use honba_api_wasm::analytics::{format_metric_value, MetricKind, MetricProvenance, RibbonMetric};
use leptos::prelude::*;

/// fe's `PROVENANCE_STYLE` — the chip label per provenance (the strings the
/// UI shows; `MetricProvenance::as_str` keeps the wire spelling for folds).
fn chip_label(provenance: MetricProvenance) -> &'static str {
    match provenance {
        MetricProvenance::Wire => "reported",
        MetricProvenance::Derived => "derived",
        MetricProvenance::Unavailable => "n/a",
    }
}

/// The provenance chip's CSS modifier (reuses the label spelling; only
/// `n/a` differs, because its chip is styled `unavailable`).
fn chip_class(provenance: MetricProvenance) -> &'static str {
    match provenance {
        MetricProvenance::Wire => "reported",
        MetricProvenance::Derived => "derived",
        MetricProvenance::Unavailable => "unavailable",
    }
}

/// fe's `toneFor`: unavailable reads muted; drawdown always reads bearish
/// (the wire reports it as a positive fraction); percent and currency signs
/// drive bull/bear; everything else stays ink.
fn tone_class(metric: &RibbonMetric) -> &'static str {
    if metric.provenance == MetricProvenance::Unavailable || metric.value.is_none() {
        return "sim-ribbon__value--muted";
    }
    let value = metric.value.unwrap_or_default();
    match metric.kind {
        MetricKind::Drawdown => "sim-ribbon__value--pnl-loss",
        MetricKind::Percent | MetricKind::Currency => {
            if value >= 0.0 {
                "sim-ribbon__value--pnl-gain"
            } else {
                "sim-ribbon__value--pnl-loss"
            }
        }
        _ => "",
    }
}

/// The ten-metric ribbon under the run header (`MetricsRibbon`).
#[component]
pub fn MetricsRibbon(
    /// The run's ribbon metrics, derived per `store.analytics_bundle`.
    #[prop(into)]
    metrics: Signal<Vec<RibbonMetric>>,
) -> impl IntoView {
    view! {
        <div class="sim-ribbon" data-testid="metrics-ribbon">
            {move || {
                metrics
                    .get()
                    .into_iter()
                    .map(|metric| {
                        let provenance_label = chip_label(metric.provenance);
                        let chip = format!("sim-ribbon__chip--{}", chip_class(metric.provenance));
                        let tone = tone_class(&metric);
                        let value_class = if tone.is_empty() {
                            "sim-ribbon__value".to_string()
                        } else {
                            format!("sim-ribbon__value {tone}")
                        };
                        view! {
                            <div class="sim-ribbon__cell" title=metric.note>
                                <div class="sim-ribbon__label-row">
                                    <span class="sim-ribbon__label">{metric.label}</span>
                                    <span class=chip>{provenance_label}</span>
                                </div>
                                <div class=value_class>
                                    {format_metric_value(metric.value, metric.kind)}
                                </div>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
