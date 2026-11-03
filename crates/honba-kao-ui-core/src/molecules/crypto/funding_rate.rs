//! `HfFundingRateTile` — crypto funding rate telemetry tile.

use leptos::prelude::*;

#[component]
pub fn HfFundingRateTile(
    #[prop(into)] rate: Signal<f64>, // annualized %
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let color = Signal::derive(move || if rate.get() >= 0.0 { "text-emerald-400" } else { "text-rose-400" });
    let base = "hf-funding-rate flex flex-col gap-1 p-2 rounded border border-border-soft bg-surface-base text-xs";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="text-text-muted">"Funding Rate (annualized)"</span>
            <div class="flex items-baseline gap-1">
                <span class=move || color.get()>{move || format!("{:.2}%", rate.get())}</span>
                <span class="text-text-muted">"/yr"</span>
            </div>
        </div>
    }
}
