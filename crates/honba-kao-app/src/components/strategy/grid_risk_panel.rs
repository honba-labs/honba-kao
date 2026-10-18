//! Grid / risk parameter panel. Mirrors
//! `web/src/components/strategy/GridRiskPanel.tsx`.

use leptos::prelude::*;

use crate::components::ui::{HfPanel, HfPanelHeader};
use crate::utils::currency;

#[component]
pub fn GridRiskPanel() -> impl IntoView {
    let levels = RwSignal::new(6i64);
    let step_pct = RwSignal::new(1.5f64);
    let capital = 250_000.0;

    let per_level = move || capital / levels.get() as f64;

    view! {
        <HfPanel>
            <HfPanelHeader title="Grid & risk" />

            <div class="hfui-panel__content form-grid">
                <label class="auth-field">
                    <span>"Grid levels"</span>
                    <input
                        type="number"
                        min="2"
                        max="20"
                        prop:value=move || levels.get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<i64>() {
                                levels.set(v.clamp(2, 20));
                            }
                        }
                    />
                </label>

                <label class="auth-field">
                    <span>"Step (%)"</span>
                    <input
                        type="number"
                        step="0.1"
                        prop:value=move || format!("{:.1}", step_pct.get())
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                step_pct.set(v.clamp(0.1, 10.0));
                            }
                        }
                    />
                </label>
            </div>

            <div class="hfui-panel__content summary-row">
                <span>"Capital per level"</span>
                <strong class="tnum">{move || currency(per_level())}</strong>
            </div>

            <div class="hfui-panel__content summary-row">
                <span>"Max drawdown guard"</span>
                <strong class="tnum">
                    {move || currency(capital * (step_pct.get() * levels.get() as f64 / 100.0))}
                </strong>
            </div>
        </HfPanel>
    }
}
