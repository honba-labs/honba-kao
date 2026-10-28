//! `HfPcrMeter` — put/call ratio meter.

use leptos::prelude::*;

#[component]
pub fn HfPcrMeter(
    #[prop(into)] pcr: Signal<f64>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let level = Signal::derive(move || {
        let p = pcr.get();
        if p > 1.2 { ("Bearish", "text-rose-400") } else if p < 0.8 { ("Bullish", "text-emerald-400") } else { ("Neutral", "text-text-muted") }
    });
    let base = "hf-pcr-meter flex flex-col items-center gap-1 p-2 rounded border border-border-soft bg-surface-base text-xs";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="text-text-muted">"PCR"</span>
            <div class="font-mono text-text-high">{move || format!("{:.2}", pcr.get())}</div>
            <span class=move || level.get().1>{level.get().0}</span>
        </div>
    }
}
