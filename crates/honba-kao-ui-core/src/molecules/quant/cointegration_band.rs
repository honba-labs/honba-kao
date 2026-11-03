//! `HfCointegrationBand` — cointegration / Z-score band for pair trading.

use leptos::prelude::*;

#[component]
pub fn HfCointegrationBand(
    #[prop(into)] z_score: Signal<f64>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let state = Signal::derive(move || {
        let z = z_score.get();
        if z > 2.0 { ("Short Spread", "text-rose-400") } else if z < -2.0 { ("Long Spread", "text-emerald-400") } else { ("Neutral", "text-text-muted") }
    });
    let base = "hf-cointegration-band flex flex-col items-center gap-1 p-2 rounded border border-border-soft bg-surface-base text-xs";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="text-text-muted">"Z-Score"</span>
            <span class=move || format!("{} font-mono text-text-high", state.get().1)>{move || format!("{:.2}", z_score.get())}</span>
            <span class="text-[10px]">{state.get().0}</span>
        </div>
    }
}
