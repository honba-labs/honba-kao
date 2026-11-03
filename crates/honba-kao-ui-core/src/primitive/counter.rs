//! `HfCounter` — monospace animated counter for active trades, open orders, or fills.
//! Ported from `secured-finance-app` atom `Counter`.

use leptos::prelude::*;

#[component]
pub fn HfCounter(
    #[prop(into)] value: Signal<usize>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-counter flex flex-col items-center gap-0.5";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="hf-counter-value font-mono font-bold text-text-high">
                {move || value.get().to_string()}
            </span>
            {label.map(|l| view! { <span class="hf-counter-label text-[10px] text-text-muted uppercase tracking-wider">{l}</span> })}
        </div>
    }
}
