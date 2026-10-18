//! Trading planner: staged entries with per-leg triggers.

use leptos::prelude::*;

use crate::components::ui::{ButtonVariant, HfButton, HfPanel, HfPanelHeader};
use crate::types::PlanLeg;
use crate::utils::currency;

#[component]
pub fn TradingPlanning() -> impl IntoView {
    let legs = RwSignal::new(vec![
        PlanLeg {
            label: "Entry tranche 1",
            trigger: "Break above opening range high",
            size_pct: 30.0,
            enabled: true,
        },
        PlanLeg {
            label: "Entry tranche 2",
            trigger: "+0.5% momentum confirmation",
            size_pct: 30.0,
            enabled: true,
        },
        PlanLeg {
            label: "Add on dip",
            trigger: "Pullback to session VWAP",
            size_pct: 20.0,
            enabled: false,
        },
        PlanLeg {
            label: "Trailing stop",
            trigger: "1.5x ATR from high",
            size_pct: 20.0,
            enabled: true,
        },
    ]);

    let allocated = move || {
        legs.get()
            .iter()
            .filter(|l| l.enabled)
            .map(|l| l.size_pct)
            .sum::<f64>()
    };

    let notional = move || currency(1_842_650.0 * allocated() / 100.0);

    view! {
        <HfPanel>
            <HfPanelHeader title="Plan structure" counter=legs.get().len()>
                <HfButton variant=ButtonVariant::Green small=true>"Deploy plan"</HfButton>
            </HfPanelHeader>

            <div class="hfui-panel__content">
                <div class="plan-legs">
                    {move || legs.get().into_iter().enumerate().map(|(i, leg)| {
                        view! {
                            <div class="plan-leg">
                                <label class="plan-leg__toggle">
                                    <input
                                        type="checkbox"
                                        prop:checked=leg.enabled
                                        on:change=move |_| legs.update(|v| v[i].enabled = !v[i].enabled)
                                    />
                                </label>
                                <div class="plan-leg__body">
                                    <strong>{leg.label}</strong>
                                    <span class="muted small">{leg.trigger}</span>
                                </div>
                                <span class="tnum plan-leg__size">
                                    {format!("{:.0}%", leg.size_pct)}
                                </span>
                            </div>
                        }
                    }).collect_view()}
                </div>

                <div class="summary-row">
                    <span>"Allocated"</span>
                    <strong class="tnum">{move || format!("{:.0}%", allocated())}</strong>
                </div>
                <div class="summary-row">
                    <span>"Working notional"</span>
                    <strong class="tnum">{notional}</strong>
                </div>
            </div>
        </HfPanel>
    }
}
