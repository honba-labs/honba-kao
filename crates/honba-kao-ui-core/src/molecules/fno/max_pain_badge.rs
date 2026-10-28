//! `HfMaxPainBadge` — max pain strike indicator.

use leptos::prelude::*;

#[component]
pub fn HfMaxPainBadge(
    #[prop(into)] strike: f64,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-max-pain inline-flex items-center gap-1 rounded border border-amber-500/30 bg-amber-600/20 px-2 py-0.5 text-[10px] font-bold text-amber-400";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <span class=classes>
            <span class="hf-icon hf-icon-sentiment" aria_hidden="true">"⬤"</span>
            {format!("Max Pain {:.0}", strike)}
        </span>
    }
}
