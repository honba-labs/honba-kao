//! The trust badges & not-modelled assumptions — the Leptos port of
//! honba-fe `features/simulator/components/validation-badges.tsx` (port
//! phase P5).
//!
//! Reuses the ported `trustBadges` mapping (`sim::ops`) so DSR / PBO /
//! hold-out / walk-forward read identically in both workspaces, and
//! surfaces the payload's `assumptions.not_modelled` list — everything the
//! run did **not** model — rather than quietly dropping it (risk R5).

use leptos::prelude::*;

use crate::features::simulator::kernel::ops::trust_badges;

/// fe's gate `DESCRIPTIONS`, keyed by `TrustBadge::key`.
const DESCRIPTIONS: [&str; 6] = [
    // dsr
    "Deflated Sharpe Ratio: corrects for selection bias across multiple trials.",
    // pbo
    "Probability of Backtest Overfitting: chance an in-sample winner underperforms out of sample.",
    // holdout
    "Hold-out test: an independent out-of-sample partition.",
    // walk_forward
    "Walk-forward analysis: rolling out-of-sample validation.",
    // monte_carlo
    "Monte Carlo resampling of the trade sequence.",
    // cost_stress
    "Cost stress: friction tolerance under multiplied fees/slippage.",
];

fn description_of(key: &str) -> &'static str {
    match key {
        "dsr" => DESCRIPTIONS[0],
        "pbo" => DESCRIPTIONS[1],
        "holdout" => DESCRIPTIONS[2],
        "walk_forward" => DESCRIPTIONS[3],
        "monte_carlo" => DESCRIPTIONS[4],
        "cost_stress" => DESCRIPTIONS[5],
        _ => "",
    }
}

/// The trust panel (`ValidationBadges`): per-gate chips over the payload's
/// validation block, the warnings, and the not-modelled assumptions.
#[component]
pub fn ValidationBadges(
    /// The run's validation block as stored on the run state
    /// (`run.response.validation`); `None` never infers anything.
    #[prop(into)]
    validation: Signal<Option<honba_kao_client::wire::ValidationBlock>>,
    /// `assumptions.not_modelled` strings off the same response.
    #[prop(into)]
    not_modelled: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="sim-validation" data-testid="validation-badges">
            <div class="sim-validation__head">
                <span class="sim-validation__title">"Institutional trust"</span>
                {move || {
                    let passed = validation
                        .get()
                        .map(|block| block.passed);
                    passed.map(|passed| {
                        let class = if passed {
                            "sim-validation__verdict sim-validation__verdict--pass"
                        } else {
                            "sim-validation__verdict sim-validation__verdict--fail"
                        };
                        view! { <span class=class>{if passed { "PASS" } else { "FAIL" }}</span> }
                    })
                }}
            </div>

            <Show
                when=move || !trust_badges(validation.get().as_ref()).is_empty()
                fallback=move || {
                    view! {
                        <div class="sim-validation__absent">
                            "The run payload carried no validation block (DSR / PBO / hold-out / walk-forward). Nothing is inferred."
                        </div>
                    }
                }
            >
                <div class="sim-validation__grid">
                    {move || {
                        trust_badges(validation.get().as_ref())
                            .into_iter()
                            .map(|badge| {
                                let verdict_class = if badge.passed {
                                    "sim-validation__gate-verdict sim-validation__gate-verdict--pass"
                                } else {
                                    "sim-validation__gate-verdict sim-validation__gate-verdict--fail"
                                };
                                let gate_class = if badge.passed {
                                    "sim-validation__gate"
                                } else {
                                    "sim-validation__gate sim-validation__gate--fail"
                                };
                                view! {
                                    <div
                                        class=gate_class
                                        data-testid=format!("trust-badge-{}", badge.key)
                                    >
                                        <div class="sim-validation__gate-head">
                                            <span class="sim-validation__gate-label">{badge.label}</span>
                                            <span class=verdict_class>
                                                {if badge.passed { "PASS" } else { "FAIL" }}
                                            </span>
                                        </div>
                                        <span class="sim-validation__gate-note">{description_of(badge.key)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </Show>

            {move || {
                let warnings = validation
                    .get()
                    .map(|block| block.warnings)
                    .unwrap_or_default();
                if warnings.is_empty() {
                    view! { <span /> }.into_any()
                } else {
                    view! {
                        <div class="sim-validation__warnings">
                            {warnings
                                .into_iter()
                                .map(|warning| view! { <span>{format!("⚠ {warning}")}</span> })
                                .collect_view()}
                        </div>
                    }
                        .into_any()
                }
            }}

            <div class="sim-validation__assumptions">
                <div class="sim-validation__assumptions-label">"Not modelled"</div>
                {move || {
                    let items = not_modelled.get();
                    if items.is_empty() {
                        view! {
                            <span class="sim-validation__assumptions-absent">
                                "The payload listed no unmodelled assumptions."
                            </span>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="sim-validation__chips">
                                {items
                                    .into_iter()
                                    .map(|item| view! { <span class="sim-validation__chip">{item}</span> })
                                    .collect_view()}
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
