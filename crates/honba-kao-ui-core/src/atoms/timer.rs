//! `HfAuctionTimer` — monospace countdown clock for call auction uncrossing or session phase.
//! Ported from `secured-finance-app` atom `Timer`.

use leptos::prelude::*;

#[component]
pub fn HfAuctionTimer(
    #[prop(into)] seconds_remaining: Signal<i64>,
    #[prop(into, optional)] label: Option<String>,
) -> impl IntoView {
    let formatted = Signal::derive(move || {
        let secs = seconds_remaining.get().max(0);
        let m = secs / 60;
        let s = secs % 60;
        format!("{m:02}:{s:02}")
    });
    view! {
        <div class="hf-auction-timer flex items-center gap-1 text-[11px] font-mono text-text-high">
            {label.map(|l| view! { <span class="text-text-muted">{l}</span> })}
            <span>{move || formatted.get()}</span>
        </div>
    }
}
