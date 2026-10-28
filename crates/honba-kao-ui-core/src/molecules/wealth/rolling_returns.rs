//! `HfRollingReturnsVisualizer` — rolling return consistency vs benchmark.

use leptos::prelude::*;

#[component]
pub fn HfRollingReturnsVisualizer(
    #[prop(into)] fund: Signal<f64>,
    #[prop(into)] benchmark: Signal<f64>,
    #[prop(into, optional)] class: Option<String>,
) -> impl IntoView {
    let base = "hf-rolling-returns rounded-lg border border-border-soft bg-surface-base p-3 text-xs";
    let classes = match class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };
    view! {
        <div class=classes>
            <span class="text-text-muted uppercase tracking-wider text-[10px] block">"Rolling Returns"</span>
            <div class="hf-returns flex items-end gap-1 h-24 mt-2">
                <div class="hf-bar flex-grow bg-emerald-500/20 rounded-t" style=format!("height: {}%;", (fund.get().clamp(0.0, 100.0)).min(100.0)) />
                <div class="hf-bar w-px bg-emerald-500 rounded-t" style=format!("height: {}%;", (fund.get().clamp(0.0, 100.0)).min(100.0)) />
                <span class="font-mono text-text-high absolute -bottom-4 left-1/2 -translate-x-1/2">{move || format!("{:.1}%", fund.get())}</span>
            </div>
            <span class="text-[10px] text-text-muted">vs benchmark {move || format!("{:.1}%", benchmark.get())}</span>
        </div>
    }
}
