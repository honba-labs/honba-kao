//! `HfItayosePhaseCard` — call-auction (Itayose) phase timer and clearing info.

use leptos::prelude::*;

#[component]
pub fn HfItayosePhaseCard(
    #[prop(into)] phase: String,
    #[prop(into)] seconds_remaining: Signal<i64>,
) -> impl IntoView {
    let formatted = Signal::derive(move || {
        let secs = seconds_remaining.get().max(0);
        format!("{:02}:{:02}", secs / 60, secs % 60)
    });
    view! {
        <div class="hf-itayose-card rounded-lg border border-border-soft bg-surface-base p-3 text-xs">
            <span class="text-text-muted uppercase tracking-wider text-[10px]">"Call Auction (Itayose)"</span>
            <div class="hf-itayose-phase flex items-baseline gap-1 mt-1">
                <span class="font-semibold text-text-high">{phase}</span>
                <span class="font-mono text-emerald-400">{move || formatted.get()}</span>
            </div>
        </div>
    }
}
