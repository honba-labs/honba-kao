//! `HfPortfolioOverlapPill` — pairwise portfolio overlap percentage indicator.

use leptos::prelude::*;

#[component]
pub fn HfPortfolioOverlapPill(
    #[prop(into)] overlap: Signal<f64>, // 0.0 ..= 100.0
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let level = Signal::derive(move || {
        let o = overlap.get();
        if o > 50.0 { ("High", "text-rose-400") } else if o > 25.0 { ("Moderate", "text-amber-400") } else { ("Low", "text-emerald-400") }
    });
    let base = "hf-overlap-pill inline-flex items-center gap-1.5 rounded border border-border-soft bg-surface-base px-2 py-0.5 text-xs font-mono";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="text-text-muted">"Overlap"</span>
            <span class=move || level.get().1>{move || format!("{:.0}%", overlap.get())}</span>
            <span class="text-text-muted">{level.get().0}</span>
        </div>
    }
}
