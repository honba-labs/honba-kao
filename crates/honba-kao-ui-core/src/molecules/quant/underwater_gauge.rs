//! `HfUnderwaterGauge` — drawdown underwater curve gauge.

use leptos::prelude::*;

#[component]
pub fn HfUnderwaterGauge(
    #[prop(into)] drawdown_pct: Signal<f64>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let color = Signal::derive(move || if drawdown_pct.get() < 20.0 { "text-emerald-400" } else { "text-rose-400" });
    let base = "hf-underwater-gauge flex flex-col items-center gap-1 p-2 rounded border border-border-soft bg-surface-base text-xs";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="text-text-muted">"Drawdown"</span>
            <span class=move || format!("{} font-mono text-text-high", color.get())>{move || format!("{:.1}%", drawdown_pct.get())}</span>
        </div>
    }
}
